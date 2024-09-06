-- get first T1 GPU avialable from PCI-Interface
local gpu = computer.getPCIDevices(classes.GPUT1)[1]
if not gpu then
 error("No GPU T1 found!")
end

-- get first Screen-Driver available from PCI-Interface
local screen = computer.getPCIDevices(classes.FINComputerScreen)[1]
-- if no screen found, try to find large screen from component network
if not screen then
 local comp = component.findComponent(classes.Screen)[1]
 if not comp then
  error("No Screen found!")
 end
 screen = component.proxy(comp)
end

-- setup gpu
event.listen(gpu)
gpu:bindScreen(screen)
w, h = gpu:getSize()

-- clear background
gpu:setBackground(0,0,0,0)
gpu:fill(0, 0, w, h, " ", " ")

-- setup color palette
colors = {{0,0,0,0},{0,0,0,0},{1,0,0,1},{1,0,0,1},{0,1,0,1},{0,1,0,1},{0,0,1,1},{0,0,1,1},{1,1,1,1},{1,1,1,1}}

-- draw color palette
for i, color in ipairs(colors) do
 gpu:setBackground(color[1], color[2], color[3], color[4])
 gpu:setText(i - 1, h - 1, " ")
end

gpu:setBackground(1,1,1,1)

-- draw loop
isDown = false
while true do
 e, s, x, y = event.pull()
 if e == "OnMouseDown" then
  isDown = true
  -- is press on color palette, select color
  if y == h - 1 and x < #colors then
   color = colors[x + 1]
   gpu:setBackground(color[1], color[2], color[3], color[4])
  end
 elseif e == "OnMouseUp" then
  isDown = false
 elseif e == "OnMouseMove" and not (y == h - 1 and x < #colors) and isDown then
  gpu:setText(x, y, " ")
 end
 gpu:flush()
end
