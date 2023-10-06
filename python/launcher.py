import re
import fileinput
import subprocess

def run(base, max_dim, recursion_parameter = None, output_type = 'file in results'):

    print(f'Program started: Z/{base} up to dimension {max_dim}')

    # Read the file into a list of lines
    with open("../src/main.rs", "r") as file:
        lines = file.readlines()

    # Modify the lines
    for i in range(len(lines)):
        lines[i] = re.sub('use typenum::U\d+ as N', f'use typenum::U{base} as N', lines[i])
        lines[i] = re.sub(r'const DIM: Int = \d+', f'const DIM: Int = {max_dim}', lines[i])
        if recursion_parameter is not None:
            lines[i] = re.sub(r'const RECURSION_PARAMETER: usize = \d+', f'const RECURSION_PARAMETER: usize = {recursion_parameter}', lines[i])

    # Write the modified lines back to the file
    with open("../src/main.rs", "w") as file:
        file.writelines(lines)
    try:
        result = subprocess.run(["cargo", "run", "-r"], text=True, capture_output=True, cwd='../src', check=True)

        if output_type == 'file in results':
            output_txt = f'../results/txt/dim{max_dim}/Z{base}-dim-{max_dim}'
            with open(output_txt, 'w') as outfile:
                outfile.write(result.stdout)
            print("Program finished succesfully")

        elif output_type == 'string':
            print("Program finished succesfully")
            return result.stdout

    except subprocess.CalledProcessError as e:
        print(f"Error while running 'cargo run -r': {e}")
        print(f"Output (if any): {e.output}\n")
