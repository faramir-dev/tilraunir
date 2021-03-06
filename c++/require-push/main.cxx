/*
 * $ g++-10 -std=c++20 -o main main.cxx ; ./main ; echo "> $?"
 */

#include <vector>
#include <array>
#include <cassert>

namespace $ = std;

static auto
capacity(auto const& container) {
	constexpr bool has_capacity = requires{ container.capacity(); };
	constexpr bool has_size = requires{ container.size(); };

	static_assert(has_capacity || has_size);

	if constexpr (has_capacity) {
		return container.capacity();
	} else if constexpr(has_size) {
		return container.size();
	}
}

template <typename Container, typename ...Param>
	requires requires (Container &container, Param &&...param) { (container.push_back(param),...); }
void push_back (Container &container, Param &&...param) { (container.push_back(param),...); }

int
main() {
	$::vector<int> v;
	$::array<int, 10> a;

	assert(capacity(v) == 0);
	assert(capacity(a) == 10);

	push_back(v, 1, 6, 7, 2);
	//push_back(a, 1, 6, 7, 2);
}
