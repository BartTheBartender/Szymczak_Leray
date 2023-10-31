import regex as re
import subprocess


def launch(base, max_dim, recursion_parameter=None):
    print(f"Program started: Z/{base} up to dimension {max_dim}\n")

    with open("src/main.rs", "r") as file:
        lines = file.readlines()

    for i in range(len(lines)):
        lines[i] = re.sub("U\d+ as N", f"U{base} as N", lines[i])
        lines[i] = re.sub(
            r"const DIM: Int = \d+", f"const DIM: Int = {max_dim}", lines[i]
        )
        if recursion_parameter is not None:
            lines[i] = re.sub(
                r"const RECURSION_PARAMETER: usize = \d+",
                f"const RECURSION_PARAMETER: usize = {recursion_parameter}",
                lines[i],
            )

    with open("src/main.rs", "w") as file:
        file.writelines(lines)
    try:
        result = subprocess.run(
            ["cargo", "run", "--release"],
            text=True,
            capture_output=True,
            cwd=".",
            check=True,
        ).stdout

        print(result)
        print("Program finished succesfully!\n-----------------------------------")
        return result

    except subprocess.CalledProcessError as e:
        print(f"Error while running 'cargo run --release': {e}")
        print(f"Output (if any): {e.output}\n")
