---@class CameraController: Component
CameraController = { velocity = 3, shift = 3, sensitivity = 0.025 }
CameraController.__index = CameraController

function CameraController:update()
    local movement = Vec3:zeros()
    if Input.getKeyHeld(Keys.W) then
        movement.z = movement.z - 1
    end
    if Input.getKeyHeld(Keys.A) then
        movement.x = movement.x - 1
    end
    if Input.getKeyHeld(Keys.S) then
        movement.z = movement.z + 1
    end
    if Input.getKeyHeld(Keys.D) then
        movement.x = movement.x + 1
    end
    movement = movement:normalize()
    if Input.getKeyHeld(Keys.LeftShift) then
        movement = movement * self.shift
    end
    Transform.moveLocal(self._entity, movement * self.velocity * FrameTime())

    local x, y = Input.getCursorOffset()
    local yRotation = Vec3.zeros()
    yRotation.y = x;
    local xRotation = Vec3.zeros()
    xRotation.x = y;
    Transform.rotate(self._entity, yRotation * self.sensitivity)
    Transform.rotateLocal(self._entity, xRotation * self.sensitivity)
end

return function()
    local object = {}
    setmetatable(object, CameraController)
    return object
end
