#!/usr/bin/env python3

unsupported_constant_type = 0
type_parameters_in_arrays = 0
higher_ranked_lifetimes_and_types = 0
unsupported_constant_value = 0
iterators = 0
cast_statements_that_create_loans = 0
determining_region_of_differentiation = 0

def process_error(error_msg):
    if "unsupported constant type" in error_msg:
        unsupported_constant_type += 1
        return
    if "unsupported constant type" in error_msg:
        unsupported_constant_type += 1
        return
    raise Exception(f"How to categorize {error_msg}")
