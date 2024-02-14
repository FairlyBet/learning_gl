local spaceHolded = Input.getKey(Keys.W, Actions.Release, Modifiers.Shift() | Modifiers.Control())
if spaceHolded then
    print "Shift Control W is released"
end
