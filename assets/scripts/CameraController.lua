--- @return table
function CameraController()
    local speed = 10

    local function move(self)
        print('move')
        local input = Input { "w", "a", "s", "d" }
        local z = input.w.hold * speed - input.s.hold * speed
        local x = input.d.hold * speed - input.a.hold * speed
    end

    local function update(self)
        print('update')
        move(self)
    end

    local function onCollision(self, other)
        print(speed)
    end

    return { update = update, onCollision = onCollision }
end

return CameraController
