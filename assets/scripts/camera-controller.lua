CameraController = { velocity = 3 }

CameraController.__index = Transform
setmetatable(CameraController, CameraController)

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
    self:moveLocal(movement * FrameTime() * self.velocity)

    local rotationY = Vec3.zeros()
    local rotationX = Vec3.zeros()
    if Input.getKeyHeld(Keys.Up) then
        rotationX.x = movement.x + 60
    end
    if Input.getKeyHeld(Keys.Down) then
        rotationX.x = movement.z - 60
    end
    if Input.getKeyHeld(Keys.Right) then
        rotationY.y = movement.y - 60
    end
    if Input.getKeyHeld(Keys.Left) then
        rotationY.y = movement.y + 60
    end
    self:rotate(rotationY * FrameTime())
    self:rotateLocal(rotationX * FrameTime())

    if Input.getKey(Keys.Tab, Actions.Press) then
        print(tostring(self:getPosition()))
        print(tostring(self:getOrientation()))
    end
end

local object = {}
object.__index = CameraController

setmetatable(object, object)

return object
