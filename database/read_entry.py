import sys
import struct
import argparse

def main():
    parser = argparse.ArgumentParser(description="Read a binary Entry struct from a file.")
    parser.add_argument("filename", help="The file to read from.")
    parser.add_argument("entry_offset", type=int, help="The offset to read from (in number of entries, not bytes).")
    
    args = parser.parse_args()

    # Struct format: 
    # >   : Big-endian
    # Q   : U64 (8 bytes) - offset
    # H   : U16 (2 bytes) - header_offset
    # H   : U16 (2 bytes) - header_size
    # I   : U32 (4 bytes) - crc
    # 32s : [u8; 32] (32 bytes) - hash
    # Q   : U64 (8 bytes) - slot
    entry_format = ">Q H H I 32s Q"
    entry_size = struct.calcsize(entry_format) # Should evaluate to 56 bytes

    byte_offset = args.entry_offset * entry_size

    try:
        with open(args.filename, 'rb') as f:
            f.seek(byte_offset)
            data = f.read(entry_size)

            if len(data) < entry_size:
                print(f"Error: Reached EOF. Could not read a full entry at index {args.entry_offset}.", file=sys.stderr)
                sys.exit(1)

            # Unpack the binary data
            unpacked = struct.unpack(entry_format, data)
            
            offset_val = unpacked[0]
            header_offset = unpacked[1]
            header_size = unpacked[2]
            crc = unpacked[3]
            hash_val = unpacked[4]
            slot = unpacked[5]

            # Print the parsed entry to stdout
            print(f"--- Entry {args.entry_offset} ---")
            print(f"offset:        {offset_val}")
            print(f"header_offset: {header_offset}")
            print(f"header_size:   {header_size}")
            print(f"crc:           {crc}")
            print(f"hash:          {hash_val.hex()}")  # Convert byte array to hex string for readability
            print(f"slot:          {slot}")

    except FileNotFoundError:
        print(f"Error: File '{args.filename}' not found.", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"An error occurred: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
