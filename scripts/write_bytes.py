some_bytes = b'\xFB\xBC'
  
# Open file in binary write mode
with open("my_file.txt", "wb") as binary_file:
    # Write bytes to file
    binary_file.write(some_bytes)
    # Close file
    binary_file.close()