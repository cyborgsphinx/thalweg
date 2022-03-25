#include "Utils.hpp"

namespace thalweg
{
namespace utils
{
auto split(std::string const& s, char c) -> std::vector<std::string>
{
	std::vector<std::string> out;
	size_t start = 0;
	size_t length = s.find(c);
	while (length != std::string::npos)
	{
		out.push_back(s.substr(start, length));
		start = start + length + 1;
		length = s.substr(start).find(c);
	}
	out.push_back(s.substr(start));
	return out;
}

auto is_close(double lhs, double rhs) -> bool
{
	// smaller than expected useful resolution for latitude and longitude
	double constexpr very_small = 0.0000000001;
	return std::abs(lhs - rhs) < very_small;
}
} // namespace utils
} // namespace thalweg
