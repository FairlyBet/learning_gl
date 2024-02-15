require("assets.scripts.vector")
local pressed = Input.getKey(Keys.Space, Actions.Press)

if pressed then
    local v = Vector:zeroes()
    print(v:addNum(10):addNum(-5))
end
