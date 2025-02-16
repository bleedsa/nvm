#include <ctype.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>

#include <u.h>
#include <vec.h>
#include <str.h>
#include <vm.h>

loc_t::loc_t(const char *v) {
	auto s = sizeof(char) * strlen(v);

	var = (char*)malloc(s);
	if (var == nullptr) fatal("malloc() returned nullptr in loc_t::loc_t");

	memcpy(var, v, s);
}

loc_t::loc_t(uint32_t t) : tmp{t} {}
loc_t::loc_t(loc_t::const_t c) : cnst{c} {}

loc_t::loc_t(const loc_t& x) {
	println("type: %d", x.type);
	type = x.type;
	if (type == VAR) {
		auto s = sizeof(char) * strlen(x.var);
		var = (char*)malloc(s);
		memcpy(var, x.var, s);
	}
}

auto loc_t::to_str() -> Str {
	char *s;
	int i;
	auto r = Str();

	switch (type) {
	case VAR:
		r.append(var);
		break;

	case TMP:
		i = asprintf(&s, "%d", tmp);
		if (i == -1) goto asprintf_err;
		r.append(s);
		free(s);
		break;

	case CONST:
		i = asprintf(&s, "%f", cnst);
		if (i == -1) goto asprintf_err;
		r.append(s);
		free(s);
		break;

	case NONE:
		r.append("NONE");
		break;
	}

	return r;

asprintf_err:
	fatal("failed to alloc str with asprintf(). returned error %d", i);
}

const loc_t NIL = loc_t();

loc_t::const_t as_const(loc_t x, Vec<Bind> v) {
	switch (x.type) {
	case VAR:
		return (loc_t::const_t)v[v.find([&](Bind *b) {
			return b->x.var == x.var;
		})]->y.un();
	case TMP:
		return (loc_t::const_t)v[v.find([&](Bind *b) {
			return b->x.tmp == x.tmp;
		})]->y.un();
	case CONST:
		return x.cnst;
	case NONE:
		fatal("NIL passed to as_const()");
	}
}

auto instr_t::to_str() -> Str {
	auto r = Str();

	r.append(to.to_str());
	r.append(" = ");

	return r;
}

auto operator==(const loc_t& x, const loc_t& y) -> bool {
	if (x.type == y.type) switch (x.type) {
		case VAR:
			return strcmp(x.var, y.var) == 0;
		case TMP:
			return x.tmp == y.tmp;
		case CONST:
			return x.cnst == y.cnst;
		case NONE:
			return true;
	}
	return false;
}

auto exe(Vec<instr_t> v) -> Result<int, Str> {
	auto binds = Vec<Bind>();

	v.for_each([](instr_t *x) {
		println("instr: %s", x->to_str().to_c_str());
	});

	for (size_t i = 0; i < v.len; i++) {
		print("assigning x...");
		auto x = v.at(i);
		println("ok");

		switch (x->type) {
		case LOCAL: {
			binds.push(Pair(x->to, Option<loc_t::const_t>()));
			break;
		}

		case MOV: {
			binds[binds.find([=](Bind *b) {
				return b->x == x->to;
			})]->y = Option<loc_t::const_t>(
				as_const(x->rhs, binds)
			);
		}

		default:
			return Result<int, Str>::mkerr(Str("instr not found"));
		}
	};

	return Result<int, Str>::mkok(0);
}
