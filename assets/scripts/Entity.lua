CameraController = { velocity = 1 }

function CameraController:update()
    -- self.transform().setPosition({ x = 0, y = 0, z = 0 })
    print "update"
end

local object = {}

setmetatable(object, { __index = CameraController })

object:update()
