/*
 * $ g++ -std=c++17 -o read_whole_file read_whole_file.cpp
 * $ ./read_whole_file 001.txt
 * 
 */

#include <sys/types.h>
#include <sys/stat.h>
#include <unistd.h>
#include <errno.h>

#include <filesystem>
#include <fstream>
#include <iostream>
#include <sstream>
#include <map>
#include <unordered_map>
#include <vector>

namespace $ = std;
namespace fs = $::filesystem;

auto read_whole_file(fs::path const& fp) {
	auto const fs = fs::file_size(fp);

	if (::size_t(fs) != fs)
		throw $::system_error(EFBIG, $::system_category());

	$::fstream f;
	f.exceptions($::ifstream::failbit | $::ifstream::badbit | $::ifstream::eofbit);
	f.open(fp);

	std::vector<$::byte> buffer(fs);
	f.read(reinterpret_cast<char*>(buffer.data()), buffer.size());

	return buffer;
}

using CSVLine = $::vector<$::string>;
using CSV = std::vector<CSVLine>;

CSV read_csv(fs::path const& file_path) {
	auto parse_line = [] (std::string const& line) {
		$::vector<std::string> ret;
		$::string temp;
		$::stringstream ss(line);
		while (getline(ss, temp, ',')) {
			ret.push_back(temp);
		}
		return ret;
	};

	CSV csv;
	$::ifstream fs(file_path);
	$::string line;
	while (getline(fs, line)) {
		csv.push_back(parse_line(line));
	}

	return csv;
}

int
main(int argc, char *argv[]) {
	$::map<$::string, int, $::less<>> m;

#if 0 
	for (int i = 1; i < argc; ++i) {
		const fs::path fp = argv[i];
		auto const file_content = read_whole_file(fp);
		$::string_view w(reinterpret_cast<char const*>(file_content.data()), file_content.size());
		auto const it = m.find(w);
		// $::cerr << "fn:[" << fn << "] -> [" << std::string(read_whole_file(fn)) << "]" << $::endl;
	}
#endif

	for( auto const& [key, val] : m) {
		std::cout << key << '\t' << val << std::endl;
	}

	for (int i = 1; i <argc; ++i) {
		fs::path fpath(argv[i]);
		auto const csv = read_csv(fpath);
		for (auto const& csv_line: csv) {
			bool first = true;
			for (auto const& item: csv_line) {
				if (!first)
					$::cout << '|';
				first = false;
				$::cout << item;
			}
			$::cout << "\n";
		}
	}

	return 0;
}