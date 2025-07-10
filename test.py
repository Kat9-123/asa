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