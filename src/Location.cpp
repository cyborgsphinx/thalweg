#include "Location.hpp"

namespace thalweg
{

auto operator==(Location const& lhs, Location const& rhs) -> bool
{
	return lhs.coord == rhs.coord && lhs.depth == rhs.depth;
}

auto operator<<(std::ostream& os, Location const& value) -> std::ostream&
{
	os << "Location:{coord:" << value.coord << ", depth:" << value.depth << "}";
	return os;
}

}