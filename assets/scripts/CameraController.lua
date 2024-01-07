local speed

local function move(self)
    print('move')
    print(speed)
    -- local input = Input { 'w', 'a', 's', 'd' }
    -- local z = input.w.hold * speed - input.s.hold * speed
    -- local x = input.d.hold * speed - input.a.hold * speed
end

local function update(self)
    print('update')
    move(self)
end

local function start()
    speed = 10
end

return { update = update, start = start }
