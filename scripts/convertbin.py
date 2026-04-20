# convert integer byte value to binary
def generateBinary(byte_value):
    # convert to binary and remove '0b' prefix
    binary = bin(byte_value)[2:]
    # pad with leading zeros to make it 8 bits
    padded_binary = binary.zfill(8)
    return padded_binary

def generateHex(byte_value):
    # convert to hex and remove '0x' prefix
    hex_value = hex(byte_value)[2:]
    return hex_value

def readFile(filename):
    try:
        with open(filename, 'rb') as file:
            content = file.read()
            return content
    except FileNotFoundError:
        print(f"File {filename} not found.")
        return None

# accept command line arguments
import sys
if len(sys.argv) > 1:
    filename = sys.argv[1]
    content = readFile(filename)
    if len(sys.argv) > 2 and sys.argv[2] == '--hex':
        for byte in content:
            print(generateHex(byte), end=' ')
    else:
        for byte in content:
            print(generateBinary(byte), end=' ')

    print()

else:
    print("Please provide a file name as an argument.")
