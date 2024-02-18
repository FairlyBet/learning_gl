require "assets.scripts.vector"

local object = {}

function object:update()
    if Input.getKey(Keys.Space, Actions.Press) then
        print(self:getPosition())
    end

    if Input.getKeyHolded(Keys.W) then
        self:move(Vector:new(1, 1, 1) * frameTime())
    end
end

setmetatable(object, { __index = Transform })

return object
