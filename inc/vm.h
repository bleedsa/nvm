enum instr_type_t {
	LOCAL,
	ADD,
	SUB,
	MOV,
};

enum loc_type_t {
	VAR,
	TMP,
	CONST,
	NONE,
};

struct loc_t {
	using const_t = double;
	loc_type_t type;
	union {
		char *var;
		uint32_t tmp;
		const_t cnst;
	};

	inline loc_t() {
		type = NONE;
	}

	loc_t(const char *x);
	loc_t(uint32_t t);
	loc_t(const_t c);

	loc_t(const loc_t& x);

	inline ~loc_t() {
		if (type == VAR) free(var);
	}

	Str to_str();

};

using Bind = Pair<loc_t, Option<loc_t::const_t>>;

loc_t::const_t as_const(loc_t x, Vec<Bind> v);

bool operator==(const loc_t& x, const loc_t& y);

/** three addr instr where loc = loc op loc */
struct instr_t {
	instr_type_t type;
	loc_t to;
	loc_t lhs;
	loc_t rhs;

	instr_t(instr_type_t ty, loc_t to, loc_t lh, loc_t rh)
		: type{ty}, to{to}, lhs{lh}, rhs{rh} {}

	Str to_str();
};

Result<int, Str> exe(Vec<instr_t> x);

extern const loc_t NIL;
