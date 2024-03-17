LightRotation = {}
LightRotation.__index = LightRotation

function LightRotation:update()
    if Input.getKeyHeld(Keys.Down) then
        local rotation = Vec3.new(20, 0, 0) * FrameTime()
        Transform.rotate(self._entity, rotation)
    end
    if Input.getKeyHeld(Keys.Up) then
        local rotation = Vec3.new(-20, 0, 0) * FrameTime()
        Transform.rotate(self._entity, rotation)
    end
    if Input.getKeyHeld(Keys.Right) then
        local rotation = Vec3.new(0, 20, 0) * FrameTime()
        Transform.rotate(self._entity, rotation)
    end
    if Input.getKeyHeld(Keys.Left) then
        local rotation = Vec3.new(0, -20, 0) * FrameTime()
        Transform.rotate(self._entity, rotation)
    end
end

return function()
    local lr = {}
    setmetatable(lr, LightRotation)
    return lr
end
