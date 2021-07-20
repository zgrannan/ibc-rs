#!/usr/bin/env python3

from sys import argv, stdin

def get_line_num(index, contents):
    return contents[0:index].count("\n")

def countFNs():
    filename = argv[1]
    contents = open(filename).read()
    current_index = 0
    while True:
        fn_index = contents.find(" fn ", current_index)
        if fn_index == -1:
            return
        is_test = contents.splitlines()[get_line_num(fn_index, contents) - 1].find("#[test]") != -1
        if is_test:
            current_index = fn_index + 4
            continue
        semicolon_index = contents.find(";", fn_index)
        first_brace_index = contents.find("{", fn_index)
        if semicolon_index != -1 and semicolon_index <= first_brace_index:
            current_index = semicolon_index
            continue
        name = contents[fn_index+4:contents.find("(", fn_index)]
        if fn_index == -1:
            raise "What?"
        braces = 0
        current_index = first_brace_index
        while braces >= 0:
            next_open = contents.find("{", current_index + 1)
            next_closed = contents.find("}", current_index + 1)
            if next_open != -1 and next_open < next_closed:
                braces += 1
                current_index = next_open
            else:
                braces -= 1
                current_index = next_closed
        num_chars = current_index - first_brace_index
        print(f"{num_chars} {filename}:{name}")

def trust(args):
    [filename, line_number_str, _] = args
    contents = open(filename).read()
    lines = contents.splitlines()
    line_number = int(line_number_str)
    if contents.find("use prusti_contracts") == -1:
        lines = lines[0:2] + [ "use prusti_contracts::*;" ] + lines[2:]
        line_number += 1
    current_line_number = line_number
    while current_line_number > 0:
        if lines[current_line_number].find("fn ") != -1:
            # print(f"Found fn on line {current_line_number}")
            if lines[current_line_number -1].find("#[trusted]") != -1:
                return
            lines = lines[0:current_line_number] + [ "#[trusted]" ] + lines[current_line_number:]
            break
        current_line_number -= 1
    else:
        raise "Couldn't find function start!"
    output = "\n".join(lines) + "\n"
    open(filename, 'w').write(output)

if argv[1] == "trust":
    trust(argv[2].split(":"))
