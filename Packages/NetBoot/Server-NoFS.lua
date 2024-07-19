--[[ Net-Boot Server ]]--

-- Configuration
local netBootPort = 8
local netBootPrograms = {}

netBootPrograms["counter.lua"] = [[
    function netBootReset()
        print("Net-Boot Restart Cleanup!")
    end
    
    local counter = 0
    while true do
        event.pull(1)
        counter = counter + 1
        print("Counter:", counter)
        if counter > 10 then
            error("meep")
        end
    end
]]

local netBootFallbackProgram = [[
    print("Invalid Net-Boot-Program: Program not found!")
    event.pull(5)
    computer.reset()
]]

-- Setup Network
local net = computer.getPCIDevices(classes.NetworkCard)[1]
if not net then
    error("Failed to start Net-Boot-Server: No network card found!")
end
net:open(netBootPort)
event.listen(net)

-- Reset all related Programs
for programName in pairs(netBootPrograms) do
    net:broadcast(netBootPort, "reset", programName)
    print("Broadcasted reset for Program \"" .. programName .. "\"")
end

-- Serve Net-Boot
while true do
    local e, _, s, p, cmd, arg1 = event.pull()
    if e == "NetworkMessage" and p == netBootPort then
        if cmd == "getEEPROM" then
            print("Program Request for \"" .. arg1 .. "\" from \"" .. s .. "\"")
            local code = netBootPrograms[arg1] or netBootFallbackProgram
            net:send(s, netBootPort, "setEEPROM", arg1, code)
        end
    end
end
