require "assets.scripts.vector"

local pressed = Input.getKey(Keys.Space, Actions.Press)

if pressed then
    print(Vector:new(10, 20, 30) * frameTime())
end
