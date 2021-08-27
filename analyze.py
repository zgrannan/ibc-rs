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
    is_cfg_attr = get_attr_fn_name(field) == "cfg_attr"
    if not is_cfg_attr:
        return False
    return is_trusted_attr_args(field["args"])

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
    return variant == "Use" or variant == "Const" or variant == "Struct" or variant == "TyAlias"

def visit(node):
    variant = node["kind"]["variant"]
    if should_skip(variant):
        return
    if variant == "Impl":
        for child in node["kind"]["fields"][0]["items"]:
            # print(f"Visit {child}")
            visit(child)
        return
    if variant == "Mod":
        name = node["ident"]["name"]
        is_test = check_is_test(node)
        # print(f"Mod {name} {is_test}")
        if is_test:
            return
        for child in node["kind"]["fields"][1]["fields"][0]:
            # print(f"Visit Mod {child}")
            visit(child)
        return
    if variant == "Fn":
        name = node["ident"]["name"]
        is_trusted = check_is_trusted(node)
        print(f"Function {name}: {is_trusted}")
        return
    raise Exception(f"Unknown AST node type {variant}")

ast = json.loads(sys.stdin.read())
for node in ast["items"]:
    visit(node)
