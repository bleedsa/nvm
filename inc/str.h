struct Str {
	char *buf;
	size_t i;
	size_t len;

	Str();
	Str(const char *x);
	~Str();

	Str(const Str& x);
	const Str& operator=(const Str& x);

	inline char *to_c_str() {
		return buf;
	}

	void resize(size_t x);
	void push(char x);

	void append(const char *x);
	void append(Str x);
};
