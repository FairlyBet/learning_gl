require "assets.scripts.vector"

CameraController = { velocity = 10 }

function CameraController:update()
    if Input.getKeyHeld(Keys.W) then
        Transform.move(self, Vector:new(0, 0, -5) * frameTime())
    end

    if Input.getKey(Keys.Space, Actions.Press) then
        print(Transform.position(self))
    end
end
