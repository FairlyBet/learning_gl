CameraController = { velocity = 2 }

CameraController.__index = Transform
setmetatable(CameraController, CameraController)

function CameraController:update()
    if Input.getKey(Keys.Space, Actions.Press, Modifiers.Control | Modifiers.Shift) then
        print(self:getPosition())
    end
end

local object = {}
object.__index = CameraController
setmetatable(object, object)

return object
