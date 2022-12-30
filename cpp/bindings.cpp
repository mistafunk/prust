#include "bindings.h"

#include "prt/API.h"

#include <map>
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

using HandlerHolder = std::map<void*, std::unique_ptr<RustLogHandlerBinding>>;
HandlerHolder logHandlerHolder;

} // namespace

void ffi_add_log_handler(AbstractLogHandlerBinding* logHandler) {
	auto [handlerIt, done] = logHandlerHolder.emplace(logHandler->context, std::make_unique<RustLogHandlerBinding>(logHandler));
	prt::addLogHandler(handlerIt->second.get());
};

void ffi_remove_log_handler(AbstractLogHandlerBinding* logHandler) {
	auto it = logHandlerHolder.find(logHandler->context);
	if (it != logHandlerHolder.end()) {
		prt::removeLogHandler(it->second.get());
	}
	delete logHandler;
}

void RustLogHandlerBinding::handleLogEvent(const wchar_t* msg, prt::LogLevel /*level*/) {
	std::string nMsg = toUTF8FromUTF16(msg);
	(*mBinding->handle_log_event)(mBinding->context, nMsg.c_str());
}

const prt::LogLevel* RustLogHandlerBinding::getLevels(size_t* count) {
	*count = prt::LogHandler::ALL_COUNT;
	return prt::LogHandler::ALL;
}

void RustLogHandlerBinding::getFormat(bool* dateTime, bool* level) {
	*dateTime = false;
	*level = false;
}
