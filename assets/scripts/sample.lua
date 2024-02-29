LightRotation = {}
LightRotation.__index = LightRotation

function LightRotation:update()
    if Input.getKeyHeld(Keys.L) then
        local rotation = Vec3.new(20, 0, 0) * FrameTime()
        Transform.rotate(self, rotation)
    end
    if Input.getKeyHeld(Keys.P) then
        local rotation = Vec3.new(-20, 0, 0) * FrameTime()
        Transform.rotate(self, rotation)
    end
end

local lr = {}

setmetatable(lr, LightRotation)

return lr
