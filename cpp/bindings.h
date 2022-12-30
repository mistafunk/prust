#pragma once

#include "prt/LogHandler.h"
#include "prt/LogLevel.h"
#include "prt/StringUtils.h"

#include <memory>

extern "C" {

struct AbstractLogHandlerBinding {
	void (*handle_log_event)(void* ctx, const char* msg);
	void* context;
};

void ffi_add_log_handler(AbstractLogHandlerBinding* logHandler);
}

class RustLogHandlerBinding : public prt::LogHandler {
public:
	explicit RustLogHandlerBinding(AbstractLogHandlerBinding* binding) : mBinding{binding} {}
	~RustLogHandlerBinding() = default;

	void handleLogEvent(const wchar_t* msg, prt::LogLevel level) override;
	const prt::LogLevel* getLevels(size_t* count) override;
	void getFormat(bool* dateTime, bool* level) override;

private:
	std::unique_ptr<AbstractLogHandlerBinding> mBinding;
};