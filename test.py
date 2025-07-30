import random
for i in range(50):
    a = random.randint(-0x8000, 0x7FFF)
    b = random.randint(a, 0x7FFF)
    print(f"!TestJumpIfLe {a} {b}")

exit()
result = 0
val = 0b0110_0001_0110_1001
print(val)
for i in range(16):
    print(bin(val))
    result <<= 1
    if val < 0x8000 or val == 0:
        result += 1
    
    val <<= 1
    val &= (0xFFFF)

print()
print(result)
print(0b1001_1110_1001_0110)