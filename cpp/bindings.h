#pragma once

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

extern "C" {

struct AttributeMapWrapper {
	int32_t dummy;
};

struct AttributeMapBuilderWrapper {
	int32_t dummy;
};

struct ResolveMapWrapper {
	int32_t dummy;
};

struct ResolveMapBuilderWrapper {
	int32_t dummy;
};

struct EncoderInfoWrapper {
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

// AttributeMapBuilder functions
AttributeMapBuilderWrapper* ffi_create_attribute_map_builder();
void ffi_destroy_attribute_map_builder(AttributeMapBuilderWrapper* builder);
prt::Status ffi_attribute_map_builder_set_string(AttributeMapBuilderWrapper* builder, const char* key, const char* value);
prt::Status ffi_attribute_map_builder_set_float(AttributeMapBuilderWrapper* builder, const char* key, double value);
prt::Status ffi_attribute_map_builder_set_int(AttributeMapBuilderWrapper* builder, const char* key, int32_t value);
prt::Status ffi_attribute_map_builder_set_bool(AttributeMapBuilderWrapper* builder, const char* key, bool value);
const AttributeMapWrapper* ffi_attribute_map_builder_create_map(AttributeMapBuilderWrapper* builder);

// ResolveMapBuilder functions
ResolveMapBuilderWrapper* ffi_create_resolve_map_builder();
void ffi_destroy_resolve_map_builder(ResolveMapBuilderWrapper* builder);
prt::Status ffi_resolve_map_builder_add_entry(ResolveMapBuilderWrapper* builder, const char* key, const char* value);
const ResolveMapWrapper* ffi_resolve_map_builder_create_map(ResolveMapBuilderWrapper* builder);

// EncoderInfo/DecoderInfo functions
const char* const* ffi_list_encoder_ids(size_t* count);
const char* const* ffi_list_decoder_ids(size_t* count);
const EncoderInfoWrapper* ffi_create_encoder_info(const char* encoder_id);
void ffi_destroy_encoder_info(const EncoderInfoWrapper* encoder_info);
const char* ffi_encoder_info_get_id(const EncoderInfoWrapper* encoder_info);
const char* ffi_encoder_info_get_name(const EncoderInfoWrapper* encoder_info);
const char* ffi_encoder_info_get_description(const EncoderInfoWrapper* encoder_info);

// StringUtils functions
prt::Status ffi_string_utils_percent_encode(const char* input, char* output, size_t* size);

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