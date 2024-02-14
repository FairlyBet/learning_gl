Input = {}

---@param key function
---@param action function
---@param modifiers? integer
---@return boolean
function Input.getKey(key, action, modifiers) return false end

---@param key function
---@return boolean
function Input.getKeyHolded(key) return false end

-- function Input.getMouseButton(button, action) end

Keys = {}

function Keys.Space() end

function Keys.Apostrophe() end

function Keys.Comma() end

function Keys.Minus() end

function Keys.Period() end

function Keys.Slash() end

function Keys.Num0() end

function Keys.Num1() end

function Keys.Num2() end

function Keys.Num3() end

function Keys.Num4() end

function Keys.Num5() end

function Keys.Num6() end

function Keys.Num7() end

function Keys.Num8() end

function Keys.Num9() end

function Keys.Semicolon() end

function Keys.Equal() end

function Keys.A() end

function Keys.B() end

function Keys.C() end

function Keys.D() end

function Keys.E() end

function Keys.F() end

function Keys.G() end

function Keys.H() end

function Keys.I() end

function Keys.J() end

function Keys.K() end

function Keys.L() end

function Keys.M() end

function Keys.N() end

function Keys.O() end

function Keys.P() end

function Keys.Q() end

function Keys.R() end

function Keys.S() end

function Keys.T() end

function Keys.U() end

function Keys.V() end

function Keys.W() end

function Keys.X() end

function Keys.Y() end

function Keys.Z() end

function Keys.LeftBracket() end

function Keys.Backslash() end

function Keys.RightBracket() end

function Keys.GraveAccent() end

function Keys.World1() end

function Keys.World2() end

function Keys.Escape() end

function Keys.Enter() end

function Keys.Tab() end

function Keys.Backspace() end

function Keys.Insert() end

function Keys.Delete() end

function Keys.Right() end

function Keys.Left() end

function Keys.Down() end

function Keys.Up() end

function Keys.PageUp() end

function Keys.PageDown() end

function Keys.Home() end

function Keys.End() end

function Keys.CapsLock() end

function Keys.ScrollLock() end

function Keys.NumLock() end

function Keys.PrintScreen() end

function Keys.Pause() end

function Keys.F1() end

function Keys.F2() end

function Keys.F3() end

function Keys.F4() end

function Keys.F5() end

function Keys.F6() end

function Keys.F7() end

function Keys.F8() end

function Keys.F9() end

function Keys.F10() end

function Keys.F11() end

function Keys.F12() end

function Keys.F13() end

function Keys.F14() end

function Keys.F15() end

function Keys.F16() end

function Keys.F17() end

function Keys.F18() end

function Keys.F19() end

function Keys.F20() end

function Keys.F21() end

function Keys.F22() end

function Keys.F23() end

function Keys.F24() end

function Keys.F25() end

function Keys.Kp0() end

function Keys.Kp1() end

function Keys.Kp2() end

function Keys.Kp3() end

function Keys.Kp4() end

function Keys.Kp5() end

function Keys.Kp6() end

function Keys.Kp7() end

function Keys.Kp8() end

function Keys.Kp9() end

function Keys.KpDecimal() end

function Keys.KpDivide() end

function Keys.KpMultiply() end

function Keys.KpSubtract() end

function Keys.KpAdd() end

function Keys.KpEnter() end

function Keys.KpEqual() end

function Keys.LeftShift() end

function Keys.LeftControl() end

function Keys.LeftAlt() end

function Keys.LeftSuper() end

function Keys.RightShift() end

function Keys.RightControl() end

function Keys.RightAlt() end

function Keys.RightSuper() end

function Keys.Menu() end

function Keys.Unknown() end

Actions = {}

function Actions.Release()
    return 0
end

function Actions.Press()
    return 1
end

Modifiers = {}

function Modifiers.Shift()
    return 1
end

function Modifiers.Control()
    return 2
end

function Modifiers.Alt()
    return 4
end

function Modifiers.Super()
    return 8
end

function Modifiers.CapsLock()
    return 16
end

function Modifiers.NumLock()
    return 32
end
