-- require "assets.scripts.vector"

CameraController = { velocity = 2 }
CameraController.__index = CameraController

function CameraController:update()
    if Input.getKey(Keys.Space, Actions.Press) then
        -- local vec = Vec3.new(1, 2, 3)
        -- vec = vec * 10
        -- vec = vec + Vec3.new(11, 3, 0.55314)
        -- vec = vec - Vec3.new(1.11, 123, 20.12544234)
        Transform.move(self, Vec3.new(-1, -2, -3))
        print(Transform.getPosition(self))
    end
    -- if Input.getKeyHeld(Keys.W) then
    --     Transform.move(self, Vector.new(0, 0, -1) * self.velocity * frameTime())
    --     print(Transform.getPosition(self))
    -- end
end

local cc = {}

-- for i = 1, 50000000 do
--     cc[i] = i
-- end

setmetatable(cc, CameraController)

return cc
