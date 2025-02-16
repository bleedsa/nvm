#include <stdint.h>
#include <ctype.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#include <u.h>
#include <str.h>

Str::Str() {
	i = 0, len = 8, buf = (char*)malloc(sizeof(char) * len);
	if (buf == nullptr) fatal("malloc() returned nullptr in Str::Str()");
}

Str::Str(const char *x) {
	auto sl = strlen(x);
	i = 0, len = sl;
	auto s = sizeof(char) * len;

	buf = (char*)malloc(s);
	if (buf == nullptr) fatal("malloc() returned nullptr in Str::Str");

	memcpy(&buf, x, s);
}

Str::~Str() {
	free(buf);
}

#define CLONE(x) { \
	i = x.i, len = x.len; \
	auto s = sizeof(char) * len; \
	buf = (char*)malloc(s); \
	if (buf == nullptr) fatal("malloc() returned nullptr in Str::CLONE"); \
	memcpy(buf, x.buf, s); \
}

Str::Str(const Str& x) CLONE(x)

const Str& Str::operator=(const Str& x) {
	CLONE(x);
	return *this;
}

auto Str::resize(size_t x) -> void {
	assert(x > len);
	len = x, buf = (char*)realloc(buf, sizeof(char) * len);
	if (buf == nullptr) fatal("realloc() returned nullptr in Str::resize");
}

auto Str::push(char x) -> void {
	if (i >= len) resize(len * 2);
	buf[i++] = x;
}

auto Str::append(const char *x) -> void {
	auto l = strlen(x);
	while ((i + l) >= len) resize(len * 2);
	for (size_t n = 0; n < l; n++) push(x[n]);
}

auto Str::append(Str x) -> void {
	auto l = x.len;
	while ((i + l) >= len) resize(len * 2);
	for (size_t n = 0; n < l; n++) push(x.buf[n]);
}
