#include "bindings.h"

#include "prt/API.h"

#include <set>
#include <string>
#include <vector>

namespace {

template <typename inC, typename outC, typename FUNC>
std::basic_string<outC> callAPI(FUNC f, const std::basic_string<inC>& s) {
	std::vector<outC> buffer(s.size());
	size_t size = buffer.size();
	f(s.c_str(), buffer.data(), &size, nullptr);
	if (size > buffer.size()) {
		buffer.resize(size);
		f(s.c_str(), buffer.data(), &size, nullptr);
	}
	return std::basic_string<outC>{buffer.data()};
}

std::string toUTF8FromUTF16(const std::wstring& utf16String) {
	return callAPI<wchar_t, char>(prt::StringUtils::toUTF8FromUTF16, utf16String);
}

std::set<std::unique_ptr<RustLogHandlerBinding>> LogHandlerHolder;

} // namespace

void register_log_handler(AbstractLogHandlerBinding* logHandler) {
	auto [it, done] = LogHandlerHolder.emplace(std::make_unique<RustLogHandlerBinding>(logHandler));
	prt::addLogHandler((*it).get());
};

void RustLogHandlerBinding::handleLogEvent(const wchar_t* msg, prt::LogLevel /*level*/) {
	std::string nMsg = toUTF8FromUTF16(msg);
	(*mBinding->handle_log_event)(mBinding->context, nMsg.c_str());
}

const prt::LogLevel* RustLogHandlerBinding::getLevels(size_t* count) {
	*count = prt::LogHandler::ALL_COUNT;
	return prt::LogHandler::ALL;
}

void RustLogHandlerBinding::getFormat(bool* dateTime, bool* level) {
	*dateTime = true;
	*level = true;
}
