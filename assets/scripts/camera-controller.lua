require "assets.scripts.vector"

CameraController = { velocity = 2 }
CameraController.__index = CameraController

function CameraController:update()
    if Input.getKeyHeld(Keys.W) then
        Transform.move(self, Vector:new(0, 0, -1) * self.velocity * frameTime())
        print(Transform.getPosition(self))
    end
end

local cc = {}
setmetatable(cc, CameraController)

return cc
