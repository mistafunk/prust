#pragma once

#include "prt/AttributeMap.h"
#include "prt/Cache.h"
#include "prt/Callbacks.h"
#include "prt/ContentType.h"
#include "prt/FileOutputCallbacks.h"
#include "prt/InitialShape.h"
#include "prt/LogHandler.h"
#include "prt/LogLevel.h"
#include "prt/OcclusionSet.h"
#include "prt/Status.h"
#include "prt/StringUtils.h"

#include <memory>

struct PRTObjectDestroyer {
	void operator()(prt::Object const* p) {
		if (p)
			p->destroy();
	}
};

using AttributeMapUPtr = std::unique_ptr<const prt::AttributeMap, PRTObjectDestroyer>;

extern "C" {

struct AttributeMapWrapper {
	int32_t dummy;

	AttributeMapUPtr createAttributeMap() const;
};

struct ResolveMapWrapper {
	int32_t dummy;
};

struct InitialShapeWrapper {
	double const* vertexCoords;
	size_t vertexCoordsCount;
	uint32_t const* indices;
	size_t indicesCount;
	uint32_t const* faceCounts;
	size_t faceCountsCount;

	const char* ruleFile;
	const char* startRule;
	int32_t randomSeed;
	const char* name;
	const AttributeMapWrapper* attributes;
	const ResolveMapWrapper* resolveMap;

	void setAttributes(prt::InitialShapeBuilder& isb, const prt::AttributeMap* am, const prt::ResolveMap* rm) const;
};

struct AbstractCallbacksBinding {
	void* context; // the actual Rust implementation
};

prt::Status ffi_generate(const InitialShapeWrapper* const* initialShapes, size_t initialShapeCount,
                         const prt::OcclusionSet::Handle* occlusionHandles, const wchar_t* const* encoders,
                         size_t encodersCount, const prt::AttributeMap* const* encoderOptions,
                         AbstractCallbacksBinding* callbacks, prt::Cache* cache, const prt::OcclusionSet* occlSet,
                         const prt::AttributeMap* generateOptions);

} // extern "C"

class RustCallbacksBinding : public prt::SimpleOutputCallbacks {
public:
	explicit RustCallbacksBinding(AbstractCallbacksBinding* binding);
	virtual ~RustCallbacksBinding() = default;

	prt::Status generateError(size_t isIndex, prt::Status status, const wchar_t* message) override;
	prt::Status assetError(size_t isIndex, prt::CGAErrorLevel level, const wchar_t* key, const wchar_t* uri,
	                       const wchar_t* message) override;
	prt::Status cgaError(size_t isIndex, int32_t shapeID, prt::CGAErrorLevel level, int32_t methodId, int32_t pc,
	                     const wchar_t* message) override;
	prt::Status cgaPrint(size_t isIndex, int32_t shapeID, const wchar_t* txt) override;
	prt::Status cgaReportBool(size_t isIndex, int32_t shapeID, const wchar_t* key, bool value) override;
	prt::Status cgaReportFloat(size_t isIndex, int32_t shapeID, const wchar_t* key, double value) override;
	prt::Status cgaReportString(size_t isIndex, int32_t shapeID, const wchar_t* key, const wchar_t* value) override;
	prt::Status attrBool(size_t isIndex, int32_t shapeID, const wchar_t* key, bool value) override;
	prt::Status attrFloat(size_t isIndex, int32_t shapeID, const wchar_t* key, double value) override;
	prt::Status attrString(size_t isIndex, int32_t shapeID, const wchar_t* key, const wchar_t* value) override;
	prt::Status attrBoolArray(size_t isIndex, int32_t shapeID, const wchar_t* key, const bool* ptr, size_t size,
	                          size_t nRows) override;
	prt::Status attrFloatArray(size_t isIndex, int32_t shapeID, const wchar_t* key, const double* ptr, size_t size,
	                           size_t nRows) override;
	prt::Status attrStringArray(size_t isIndex, int32_t shapeID, const wchar_t* key, const wchar_t* const* ptr,
	                            size_t size, size_t nRows) override;

	bool canSeek() const override {
		return mDelegate->canSeek();
	}
	uint64_t open(const wchar_t* encoderId, const prt::ContentType contentType, const wchar_t* name,
	              StringEncoding enc = SE_NATIVE, OpenMode mode = OPENMODE_ALWAYS, prt::Status* stat = 0) override {
		return mDelegate->open(encoderId, contentType, name, enc, mode, stat);
	}
	prt::Status write(uint64_t handle, const wchar_t* string) override {
		return mDelegate->write(handle, string);
	};
	prt::Status write(uint64_t handle, const uint8_t* buffer, size_t size) override {
		return mDelegate->write(handle, buffer, size);
	};
	prt::Status seek(uint64_t handle, int64_t offset, SeekOrigin origin) override {
		return mDelegate->seek(handle, offset, origin);
	};
	uint64_t tell(uint64_t handle, prt::Status* stat = 0) override {
		return mDelegate->tell(handle, stat);
	};
	prt::Status close(uint64_t handle, const size_t* isIndices, size_t isCount) override {
		return mDelegate->close(handle, isIndices, isCount);
	};

	prt::Status openCGAError(const wchar_t* name) override {
		return mDelegate->openCGAError(name);
	};
	prt::Status openCGAPrint(const wchar_t* name) override {
		return mDelegate->openCGAPrint(name);
	};
	prt::Status openCGAReport(const wchar_t* name) override {
		return mDelegate->openCGAReport(name);
	};

	prt::Status closeCGAError() override {
		return mDelegate->closeCGAError();
	};
	prt::Status closeCGAPrint() override {
		return mDelegate->closeCGAPrint();
	};
	prt::Status closeCGAReport() override {
		return mDelegate->closeCGAReport();
	};

private:
	std::unique_ptr<AbstractCallbacksBinding> mBinding;
	std::unique_ptr<prt::FileOutputCallbacks, PRTObjectDestroyer> mDelegate;
};

extern "C" {

struct AbstractLogHandlerBinding {
	void (*handle_log_event)(void* ctx, const char* msg);
	void* context; // the actual Rust implementation
};

void ffi_add_log_handler(AbstractLogHandlerBinding* logHandler);
void ffi_remove_log_handler(AbstractLogHandlerBinding* logHandler);

} // extern "C"

class RustLogHandlerBinding : public prt::LogHandler {
public:
	explicit RustLogHandlerBinding(AbstractLogHandlerBinding* binding) : mBinding{binding} {}
	virtual ~RustLogHandlerBinding() = default;

	void handleLogEvent(const wchar_t* msg, prt::LogLevel level) override;
	const prt::LogLevel* getLevels(size_t* count) override;
	void getFormat(bool* dateTime, bool* level) override;

private:
	std::unique_ptr<AbstractLogHandlerBinding> mBinding;
};