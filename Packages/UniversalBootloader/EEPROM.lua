bootloader = {
    init = nil,
    port = 8,
    srv = "/srv",
    storageMounted = false,
}

local fs = filesystem

function bootloader:mountStorage(searchFile)
    if self.storageMounted then
        return
    end

    fs.initFileSystem("/dev")

    local devs = fs.children("/dev")
	for _, dev in pairs(devs) do
		local drive = filesystem.path("/dev", dev)
        fs.mount(drive, self.srv)
        if searchFile == nil or self:programExists(searchFile) then
            self.storageMounted = true
            return true
        end
        fs.unmount(drive)
	end

    return false
end

function bootloader:programExists(name)
    local path = filesystem.path(self.srv, name)
    return filesystem.exists(path) and filesystem.isFile(path)
end

function bootloader:loadFromStorage(name)
    if not self.storageMounted then
        self:mountStorage()
    end

    if not self:programExists(name) then
        return nil
    end

	fd = fs.open("/srv/"..name, "r")
	content = ""
	while true do
		chunk = fd:read(1024)
		if chunk == nil or #chunk == 0 then
			break
		end
		content = content .. chunk
	end
	return content
end

function bootloader:initNetBoot()
    if self.netBootInitDone then
        return
    end
    self.net = computer.getPCIDevices(classes.NetworkCard)[1]
    if not self.net then
        error("Net-Boot: Failed to Start: No Network Card available!")
    end
    self.net:open(self.port)
    event.listen(self.net)

    -- Wrap event.pull() and filter Net-Boot messages
    local og_event_pull = event.pull
    function event.pull(timeout)
        local args = {og_event_pull(timeout)}
        local e, _, s, p, cmd, programName = table.unpack(args)
        if e == "NetworkMessage" and p == self.port then
            if cmd == "reset" and programName == self.init then
                computer.log(2, "Net-Boot: Received reset command from Server \"" .. s .. "\"")
                if netBootReset then
                    pcall(netBootReset)
                end
                computer.reset()
            end
        end
        return table.unpack(args)
    end
    self.netBootInitDone = true
end

function bootloader:loadFromNetBoot(name)
    if not self.netBootInitDone then
        self:initNetBoot()
    end
    self.net:broadcast(self.port, "getEEPROM", name)
    local program = nil
    while program == nil do
        local e, _, s, p, cmd, programName, code = event.pull(30)
        if e == "NetworkMessage" and p == self.port and cmd == "setEEPROM" and programName == name then
            print("Net-Boot: Got Code for Program \"" .. name .. "\" from Server \"" .. s .. "\"")
            return code
        elseif e == nil then
            computer.log(3, "Net-Boot: Request Timeout reached! Retry...")
            break
        end
    end
    return nil
end

function bootloader:loadCode(name)
    if not self.storageMounted then
        computer.log(0, "Mounting storage")
        self:mountStorage()
    end

    local content = nil
    if self.storageMounted then
        computer.log(0, "Loading " .. name .. " from storage")
        content = self:loadFromStorage(name)
    else
        computer.log(0, "No storage available")
    end

    if not content then
        computer.log(0, "Loading " .. name .. " from net boot")
        content = self:loadFromNetBoot(name)
    end

    return content
end

function bootloader:parseModule(name)
    local content = self:loadCode(name)
    if content then
        computer.log(0, "Parsing loaded content")
        local code, error = load(content)
        if not code then
            computer.log(4, "Failed to parse " .. name .. ": " .. tostring(error))
            event.pull(2)
            computer.reset()
        end
        return code
    else
        computer.log(3, "Could not load " .. name .. ": Not found.")
        return nil
    end
end

function bootloader:loadModule(name)
    computer.log(0, "Loading " .. name .. " through the bootloader")
    local code = self:parseModule(name)
    if code then
        -- We don't really expect this to return
        computer.log(0, "Starting " .. name)
        local success, error = pcall(code)
        if not success then
            computer.log(3, error)
            event.pull(2)
            computer.reset()
        end
    else
        computer.log(4, "Failed to load module "..name)
    end
end

function bootloader:main()
    if not self.init then
        self.init = computer.getInstance().nick
    end
    if not self.init then
        computer.log(4, "No init program set")
        computer.stop()
    end
    self.init = string.gsub(self.init, "#.*$", "")
    computer.log(1, "Booting " .. self.init)
    self:loadModule(self.init)
end

bootloader:main()
