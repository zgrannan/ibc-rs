#!/usr/bin/env python3

import typing
from sys import argv, stdin

def get_line_num(index, contents):
    return contents[0:index].count("\n")

def count_chars_between_braces(first_brace_index, contents):
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
    return current_index - first_brace_index

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
        num_chars = count_chars_between_braces(first_brace_index, contents)
        print(f"{num_chars} {filename}:{name}")

def get_fn_body_lines(fn_line_number, lines):
    contents = "\n".join(lines[fn_line_number:])
    first_brace_index = contents.find("{")
    num_chars = count_chars_between_braces(first_brace_index, contents)
    start = get_line_num(first_brace_index, contents) + 1
    end = get_line_num(first_brace_index + num_chars, contents)
    return (fn_line_number + start, fn_line_number + end)

def get_enclosing_fn_line_number(line_number, lines):
    current_line_number = line_number
    while current_line_number > 0:
        if lines[current_line_number].find("fn ") != -1:
            return current_line_number
        current_line_number -= 1
    else:
        return None

def parse_arg(arg):
    args = arg.split(":")
    if len(args) != 3:
        raise Exception(f"Can't parse argument {arg}")
    return (args[0], int(args[1]) - 1)

def panic(arg: str):
    (filename, line_number) = parse_arg(arg)
    contents = open(filename).read()
    lines = contents.splitlines()
    fn_line_number = get_enclosing_fn_line_number(line_number, lines)
    if fn_line_number is None:
        raise Exception(f"Couldn't find enclosing function for {filename} at line {line_number}")
    (start, end) = get_fn_body_lines(fn_line_number, lines)
    commented = ["// " + line for line in lines[start:end] ]
    commented[0] = 'panic!("No") ' + commented[0]
    result = lines[:start] + commented + lines[end:]
    output = "\n".join(result) + "\n"
    open(filename, 'w').write(output)

def trust(arg: str):
    (filename, line_number) = parse_arg(arg)
    contents = open(filename).read()
    lines = contents.splitlines()
    if contents.find("use prusti_contracts") == -1:
        lines = lines[0:2] + [ "use prusti_contracts::*;" ] + lines[2:]
        line_number += 1
    fn_line_number = get_enclosing_fn_line_number(line_number, lines)
    if fn_line_number is None:
        print(f"Couldn't find enclosing function for {filename} at line {line_number}")
        return
    if lines[fn_line_number -1].find("#[trusted]") != -1:
        return
    lines = lines[0:fn_line_number] + [ "#[trusted]" ] + lines[fn_line_number:]
    output = "\n".join(lines) + "\n"
    open(filename, 'w').write(output)

action = trust

if len(argv) > 1 and argv[1] == "panic":
    action = panic

chunks = stdin.read().split("\nerror:")
for chunk in chunks[1:]:
    chunk_lines = chunk.splitlines()
    if len(chunk_lines) < 2:
        continue # Last Chunk
        #  raise Exception(f"Chunk {chunk} doesn't have enough lines!")
    loc_chunks = chunk_lines[1].split("--> ")
    if len(loc_chunks) <= 1:
        raise Exception(f"Chunk {chunk} doesn't have a location")
    action(loc_chunks[1])
