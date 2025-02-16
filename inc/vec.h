template<typename T>
struct Vec {
	T *buf;
	size_t i;
	size_t len;

	Vec() {
		i = 0, len = 8, buf = (T*)malloc(sizeof(T) * len);
		if (buf == nullptr)
			fatal("malloc() returned nullptr in Vec::Vec()");
	}

	~Vec() {
		free(buf);
	}

	#define CLONE(x) { \
		i = x.i, len = x.len; \
		auto s = sizeof(T) * len; \
		buf = (T*)malloc(s); \
		if (buf == nullptr) \
			fatal("malloc() returned nullptr in Vec::CLONE"); \
		memcpy(buf, x.buf, s); \
	}

	Vec(const Vec& x) {
		CLONE(x);
	}

	const Vec& operator=(const Vec& x) {
		CLONE(x);
		return *this;
	}

	T *begin() { return &buf[0]; }
	const T *begin() const { return &buf[0]; }
	T *end() { return &buf[i - 1]; }
	const T *end() const { return &buf[i - 1]; }

	T *operator[](int i) {
		return &buf[i];
	}

	void resize(size_t x) {
		assert(x > len);
		len = x, buf = (T*)realloc(buf, sizeof(T) * len);
		if (buf == nullptr)
			fatal("realloc() returned nullptr in Vec::resize");
	}

	void push(T x) {
		if (i >= len) resize(len * 2);
		buf[i] = x, i++;
	}
	
	inline T *at(size_t i) {
		return &buf[i];
	}

	template<typename F>
	inline void for_each(F f) {
		for (size_t n = 0; n < i; n++) f(&buf[n]);
	}

	template<typename F>
	inline int64_t find(F f) {
		for (size_t n = 0; n < i; n++) if (f(&buf[n])) return n;
		return -1;
	}
};
