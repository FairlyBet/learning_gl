LightRotation = { sensitivity = 0.08 }
LightRotation.__index = LightRotation

function LightRotation:update()
    local x, y = Input.getCursorOffset()
    local yRotation = Vec3.zeros()
    yRotation.y = x;
    local xRotation = Vec3.zeros()
    xRotation.x = -y;
    Transform.rotate(self._entity, yRotation * self.sensitivity)
    Transform.rotateLocal(self._entity, xRotation * self.sensitivity)
end

return function()
    local lr = {}
    setmetatable(lr, LightRotation)
    return lr
end
