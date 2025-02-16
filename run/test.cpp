#include <ctype.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdio.h>

#include <u.h>
#include <vec.h>
#include <str.h>
#include <vm.h>

int main(void) {
	auto in = Vec<instr_t>();
	in.push(instr_t(LOCAL, loc_t("x"), NIL, NIL));

	exe(in);

	return 0;
}
