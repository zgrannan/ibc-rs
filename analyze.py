#!/usr/bin/env python3

import json
import sys

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

def is_trusted_attr_args(args):
    fields = args["fields"][2]["0"]
    feature_word = fields[0][0]["fields"][0]["kind"]["fields"][0] == "feature"
    if not feature_word:
        return False
    eq = acc(fields[1][0], "fields", 0, "kind") == "Eq"
    if not eq:
        return False
    prusti_word = acc(fields[2][0], "fields", 0, "kind", "fields", 0, "symbol") == "prusti"
    return prusti_word

def is_test_attr_args(args):
    fields = args["fields"][2]["0"]
    return fields[0][0]["fields"][0]["kind"]["fields"][0] == "test"

def get_attr_fn_name(field):
    return acc(field, "path", "segments", 0, "ident", "name")

def is_trusted_attr(field):
    attr_fn_name = get_attr_fn_name(field)
    if attr_fn_name == "trusted":
        return True
    elif attr_fn_name == "cfg_attr":
        return is_trusted_attr_args(field["args"])
    else:
        return False

def is_test_attr(field):
    is_cfg_attr = get_attr_fn_name(field) == "cfg"
    if not is_cfg_attr:
        return False
    return is_test_attr_args(field["args"])

def check_is_trusted(node):
    return check_has_attr(node, is_trusted_attr)

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

result = {}

def get_name(node):
    return node["ident"]["name"]

def extend_path(path, name):
    if path == "":
        return name
    else:
        return path + "." + name

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
        full_name = extend_path(path, get_name(node))
        is_trusted = check_is_trusted(node)
        if full_name in result and result[full_name]:
            return
        else:
            result[full_name] = is_trusted
        return
    raise Exception(f"Unknown AST node type {variant}")

ast = json.loads(sys.stdin.read())
for node in ast["items"]:
    visit(node, "")

for name, trusted in result.items():
    if trusted:
        print(f"{name} skipped")
    else:
        print(f"{name} verified")
