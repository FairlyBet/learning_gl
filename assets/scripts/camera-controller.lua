CameraController = { velocity = 2 }

CameraController.__index = Transform
setmetatable(CameraController, CameraController)

function CameraController:update()
    if Input.getKey(Keys.Space, Actions.Press) then
        print("Position " .. tostring(self:getPosition()))
    end
    if Input.getKey(Keys.Tab, Actions.Press) then
        print("Orientation " .. tostring(self:getOrientation()))
    end
    if Input.getKey(Keys.R, Actions.Press) then
        self:rotate(Vec3.new(0, 90, 0))
    end
    if Input.getKey(Keys.Delete, Actions.Press) then
        DeleteObject(self)
    end
end

local object = {}
object.__index = CameraController

for i = 1, 10000000 do
    object[i] = i
end

setmetatable(object, object)

return object
