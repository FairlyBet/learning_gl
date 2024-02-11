-- CameraController = { velocity = 2, name = "abc" }

-- function CameraController:update()
--     -- self.transform().setPosition({ x = 0, y = 0, z = 0 })
--     print(self.velocity)
-- end

-- function CameraController:f()
--     self.velocity = self.velocity - 1
--     print(self.velocity)
--     -- print(self.mass)
--     print(self.name)
-- end

-- CameraController.__index = CameraController


-- print(object["update"] ~= nil)
-- require("assets.scripts.readonly")

-- function SetAddress(address, f)
--     return function(param)
--         rawset(param, "address", address)
--         f(param)
--     end
-- end

-- local _getTransform = function(arg)
-- end


-- Entity = {}

-- function Entity.new(id)
--     local entity = { id = id }
--     return entity
-- end

-- function Entity:setPosition(position)
--     assert(type(self.id), "number")
--     _getTransform({ id = self.id, x = position["x"], y = position["y"], z = position["z"] })
-- end

-- function ProvideId(id)
--     return function()
--         return id
--     end
-- end

-- local entity = Entity.new(10)


-- CameraController = {}

-- function CameraController:update()
--     self.transform.move({ 1, 0, 0 })
-- end

return 18446744073709551615
