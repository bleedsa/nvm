#define print(...) { \
	printf(__VA_ARGS__); \
	fflush(stdout); \
}

#define println(...) { \
	printf(__VA_ARGS__); \
	putc('\n', stdout); \
}

#define eprintln(...) { \
	fprintf(stderr, __VA_ARGS__); \
	putc('\n', stderr); \
}

#define fatal(...) { \
	eprintln(__VA_ARGS__); \
	exit(-1); \
}

#define assert(x) if (!(x)) fatal("assertion %s failed!", #x);

template<typename T, typename E>
struct Result {
	union {
		T ok;
		E no;
	};
	bool is_ok;

	inline static Result<T, E> mkok(T x) {
		auto r = Result();
		r.ok = x;
		r.is_ok = true;
		return r;
	}

	inline static Result<T, E> mkerr(E x) {
		auto r = Result();
		r.no = x;
		r.is_ok = false;
		return r;
	}

	Result& operator=(const Result& x) {
		is_ok = x.is_ok;
		if (is_ok) ok = x.ok;
		else no = x.no;
		return *this;
	}

	Result(const Result& x) {
		is_ok = x.is_ok;
		if (is_ok) ok = x.ok;
		else no = x.no;
	}

	Result() {}
	~Result() {}

	inline bool is() { return is_ok; }
	inline T un() { return ok; }
	inline T err() { return no; }
};

template<typename T>
struct Option {
	union {
		T i;
	};
	/* does this union have a value? */
	bool val;

	Option(T x) {
		i = x;
		val = true;
	}

	Option() {
		val = false;
	}

	Option& operator=(const Option& x) {
		val = x.val;
		if (val) i = x.i;
		return *this;
	}

	~Option() {}

	inline bool is() { return val; }
	inline T un() { return i; }
};

template<typename T>
bool operator==(const Option<T>& x, const Option<T>& y) {
	if (x.val == y.val) {
		if (x.val) return x.i == y.i;
		else return true;
	} else return false;
}

template<typename X, typename Y>
struct Pair {
	X x;
	Y y;

	Pair(X x, Y y) : x{x}, y{y} {}
	~Pair() {}

	Pair(const Pair& p) : x{p.x}, y{p.y} {}
	const Pair& operator=(const Pair& p) {
		x = p.x, y = p.y;
		return *this;
	}
};
