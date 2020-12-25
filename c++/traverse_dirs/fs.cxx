#include <filesystem>
#include <iostream>

namespace $ = std;
namespace fs = $::filesystem;

static void traverse_dir(fs::path const& fpath) {
	try {
		for (auto const& dentry: fs::directory_iterator{fpath}) {
			$::cout << dentry.path() << "\n";
			if (dentry.is_directory()) {
				traverse_dir(dentry.path());
			}
		}
	} catch (fs::filesystem_error const& err) {
		$::cout << fpath << " : " << err.what() << "\n";
	}
}

int
main(void) {
	traverse_dir(fs::path{"/"});
}
