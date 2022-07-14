#!/usr/bin/env python3

import json
import sys

VERIFIED = "VERIFIED"
VERIFIED_FAST = "VERIFIED_FAST"
SKIPPED = "SKIPPED"

def acc(node, *path):
    result = node
    for e in path:
        try:
            result = result[e]
        except TypeError as err:
            print(f"Cannot access field `{e}` of {result}")
            raise err
        except KeyError as err:
            print(f"Cannot access field `{e}` of {result}")
            raise err
    return result

def attr_arg_name_eq(args, name):
    fields = args["fields"][2]["0"]
    feature_word = fields[0][0]["fields"][0]["kind"]["fields"][0] == "feature"
    if not feature_word:
        return False
    eq = acc(fields[1][0], "fields", 0, "kind") == "Eq"
    if not eq:
        return False
    return acc(fields[2][0], "fields", 0, "kind", "fields", 0, "symbol") == name

def is_verified_fast_attr_args(args):
    return attr_arg_name_eq(args, "prusti_fast")

def is_trusted_attr_args(args):
    return attr_arg_name_eq(args, "prusti")

def is_test_attr_args(args):
    fields = args["fields"][2]["0"]
    return fields[0][0]["fields"][0]["kind"]["fields"][0] == "test"

def get_attr_fn_name(field):
    return acc(field, "path", "segments", 0, "ident", "name")

def is_trusted_attr(field):
    attr_fn_name = get_attr_fn_name(field)
    if attr_fn_name == "trusted" or attr_fn_name == "trusted_skip":
        return True
    elif attr_fn_name == "cfg_attr":
        return is_trusted_attr_args(field["args"])
    else:
        return False

def is_verified_fast_attr(field):
    attr_fn_name = get_attr_fn_name(field)
    if attr_fn_name == "cfg_attr":
        return is_verified_fast_attr_args(field["args"])
    else:
        return False

def is_test_attr(field):
    is_cfg_attr = get_attr_fn_name(field) == "cfg"
    if not is_cfg_attr:
        return False
    return is_test_attr_args(field["args"])

def get_label(node):
    if check_has_attr(node, is_trusted_attr):
        return SKIPPED
    elif check_has_attr(node, is_verified_fast_attr):
        return VERIFIED_FAST
    else:
        return VERIFIED

def check_is_test(node):
    return check_has_attr(node, is_test_attr)

def check_has_attr(node, check):
    for attr in node["attrs"]:
        if attr["kind"]["variant"] != "Normal":
            continue
        fields = attr["kind"]["fields"]
        if check(fields[0]):
            return True
    return False


def should_skip(variant):
    return variant == "Use" or variant == "Const" or variant == "Struct" or variant == "TyAlias" or variant == "Enum" or variant == "MacCall" or variant == "MacroDef"


def get_name(node):
    return node["ident"]["name"]

def extend_path(path, name):
    if path == "":
        return name
    else:
        return path + "." + name

result = {}


def visit(node, path):
    variant = node["kind"]["variant"]
    if should_skip(variant):
        return
    if variant == "Trait":
        for child in acc(node, "kind", "fields", 0, "4"):
            visit(child, extend_path(path, get_name(node)))
        return

    if variant == "Impl":
        for child in acc(node, "kind", "fields", 0, "items"):
            # print(f"Visit {child}")
            visit(child, extend_path(path, get_name(node)))
        return
    if variant == "Mod":
        name = node["ident"]["name"]
        is_test = check_is_test(node)
        if is_test:
            return
        field = acc(node, "kind", "fields", 1)
        if field == "Unloaded":
            return
        for child in acc(field, "fields", 0):
            # print(f"Visit Mod {child}")
            visit(child, extend_path(path, get_name(node)))
        return
    if variant == "Fn":
        if not node["kind"]["fields"][0]["3"]: # Interface
            return
        # print(json.dumps(node))
        # print(",")
        full_name = extend_path(path, get_name(node))
        label = get_label(node)
        char_number = node["ident"]["span"]["lo"]
        if full_name in result and result[full_name][0] == SKIPPED:
            return
        else:
            result[full_name] = (label, char_number)
        return
    raise Exception(f"Unknown AST node type {variant}")

ast = json.loads(sys.stdin.read())
# print("[")
for node in ast["items"]:
    visit(node, "")
# print("0]")

for name, (label, line_number) in result.items():
    print(f"{name} {label} {line_number}")
