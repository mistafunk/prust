#include "bindings.h"

#include "prt/API.h"
#include "prt/AttributeMap.h"
#include "prt/DecoderInfo.h"
#include "prt/EncoderInfo.h"
#include "prt/ResolveMap.h"

#include <filesystem>
#include <map>
#include <string>
#include <vector>
#include <cstring>

namespace {

using AttributeMapBuilderUPtr = std::unique_ptr<prt::AttributeMapBuilder, PRTObjectDestroyer>;
using AttributeMapUPtr = std::unique_ptr<const prt::AttributeMap, PRTObjectDestroyer>;
using EncoderInfoUPtr = std::unique_ptr<const prt::EncoderInfo, PRTObjectDestroyer>;
using InitialShapeBuilderUPtr = std::unique_ptr<prt::InitialShapeBuilder, PRTObjectDestroyer>;
using InitialShapeUPtr = std::unique_ptr<const prt::InitialShape, PRTObjectDestroyer>;
using InitialShapeNOPtrVector = std::vector<const prt::InitialShape*>;
using ResolveMapUPtr = std::unique_ptr<const prt::ResolveMap, PRTObjectDestroyer>;
using ResolveMapBuilderUPtr = std::unique_ptr<prt::ResolveMapBuilder, PRTObjectDestroyer>;

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

std::wstring toUTF16FromUTF8(const std::string& utf8String) {
	return callAPI<char, wchar_t>(prt::StringUtils::toUTF16FromUTF8, utf8String);
}

const prt::AttributeMap* createValidatedOptions(const wchar_t* encID,
                                                const prt::AttributeMap* unvalidatedOptions = nullptr) {
	const EncoderInfoUPtr encInfo(prt::createEncoderInfo(encID));
	const prt::AttributeMap* validatedOptions = nullptr;
	const prt::AttributeMap* optionStates = nullptr;
	const prt::Status s =
	        encInfo->createValidatedOptionsAndStates(unvalidatedOptions, &validatedOptions, &optionStates);
	if (optionStates != nullptr)
		optionStates->destroy();
	return (s == prt::STATUS_OK) ? validatedOptions : nullptr;
}

} // namespace

void InitialShapeWrapper::setAttributes(prt::InitialShapeBuilder& isb, const prt::AttributeMap* am,
                                        const prt::ResolveMap* rm) const {
	std::wstring wRuleFile = toUTF16FromUTF8(ruleFile);
	std::wstring wStartRule = toUTF16FromUTF8(startRule);
	std::wstring wName = toUTF16FromUTF8(name);
	isb.setAttributes(wRuleFile.c_str(), wStartRule.c_str(), randomSeed, wName.c_str(), am, rm);
}

RustCallbacksBinding::RustCallbacksBinding(AbstractCallbacksBinding* binding) : mBinding(binding) {
	auto prustTempPath = std::filesystem::temp_directory_path() / "prust";
	std::filesystem::create_directories(prustTempPath);
	mDelegate.reset(prt::FileOutputCallbacks::create(prustTempPath.wstring().c_str()));
}

prt::Status RustCallbacksBinding::generateError(size_t /*isIndex*/, prt::Status /*status*/,
                                                const wchar_t* /*message*/) {
	return prt::STATUS_OK;
}

prt::Status RustCallbacksBinding::assetError(size_t /*isIndex*/, prt::CGAErrorLevel /*level*/, const wchar_t* /*key*/,
                                             const wchar_t* /*uri*/, const wchar_t* /*message*/) {
	return prt::STATUS_OK;
}

prt::Status RustCallbacksBinding::cgaError(size_t /*isIndex*/, int32_t /*shapeID*/, prt::CGAErrorLevel /*level*/,
                                           int32_t /*methodId*/, int32_t /*pc*/, const wchar_t* /*message*/) {
	return prt::STATUS_OK;
}

prt::Status RustCallbacksBinding::cgaPrint(size_t /*isIndex*/, int32_t /*shapeID*/, const wchar_t* /*txt*/) {
	return prt::STATUS_OK;
}

prt::Status RustCallbacksBinding::cgaReportBool(size_t /*isIndex*/, int32_t /*shapeID*/, const wchar_t* /*key*/,
                                                bool /*value*/) {
	return prt::STATUS_OK;
}

prt::Status RustCallbacksBinding::cgaReportFloat(size_t /*isIndex*/, int32_t /*shapeID*/, const wchar_t* /*key*/,
                                                 double /*value*/) {
	return prt::STATUS_OK;
}

prt::Status RustCallbacksBinding::cgaReportString(size_t /*isIndex*/, int32_t /*shapeID*/, const wchar_t* /*key*/,
                                                  const wchar_t* /*value*/) {
	return prt::STATUS_OK;
}

prt::Status RustCallbacksBinding::attrBool(size_t /*isIndex*/, int32_t /*shapeID*/, const wchar_t* /*key*/,
                                           bool /*value*/) {
	return prt::STATUS_OK;
}

prt::Status RustCallbacksBinding::attrFloat(size_t /*isIndex*/, int32_t /*shapeID*/, const wchar_t* /*key*/,
                                            double /*value*/) {
	return prt::STATUS_OK;
}

prt::Status RustCallbacksBinding::attrString(size_t /*isIndex*/, int32_t /*shapeID*/, const wchar_t* /*key*/,
                                             const wchar_t* /*value*/) {
	return prt::STATUS_OK;
}

prt::Status RustCallbacksBinding::attrBoolArray(size_t /*isIndex*/, int32_t /*shapeID*/, const wchar_t* /*key*/,
                                                const bool* /*values*/, size_t /*size*/, size_t /*nRows*/) {
	return prt::STATUS_OK;
}

prt::Status RustCallbacksBinding::attrFloatArray(size_t /*isIndex*/, int32_t /*shapeID*/, const wchar_t* /*key*/,
                                                 const double* /*values*/, size_t /*size*/, size_t /*nRows*/) {
	return prt::STATUS_OK;
}

prt::Status RustCallbacksBinding::attrStringArray(size_t /*isIndex*/, int32_t /*shapeID*/, const wchar_t* /*key*/,
                                                  const wchar_t* const* /*values*/, size_t /*size*/, size_t /*nRows*/) {
	return prt::STATUS_OK;
}

prt::Status ffi_generate(const InitialShapeWrapper* const* ffiInitialShapes, size_t initialShapeCount,
                         const prt::OcclusionSet::Handle* occlusionHandles, const wchar_t* const* encoders,
                         size_t encodersCount, const prt::AttributeMap* const* /*encoderOptions*/,
                         AbstractCallbacksBinding* callbacks, prt::Cache* cache, const prt::OcclusionSet* occlSet,
                         const prt::AttributeMap* generateOptions) {
	std::vector<InitialShapeUPtr> initialShapes; // keeps the initial shapes alive
	initialShapes.reserve(initialShapeCount);
	std::vector<AttributeMapUPtr> initialShapeAttributes; // keeps the initial shape attrs alive
	initialShapeAttributes.reserve(initialShapeCount);
	std::vector<ResolveMapUPtr> resolveMaps; // keeps the initial shape resolve maps alive
	resolveMaps.reserve(initialShapeCount);

	InitialShapeNOPtrVector initialShapePtrs(initialShapeCount, nullptr);
	InitialShapeBuilderUPtr isb(prt::InitialShapeBuilder::create());
	AttributeMapBuilderUPtr amb(prt::AttributeMapBuilder::create());
	ResolveMapBuilderUPtr rmb(prt::ResolveMapBuilder::create());
	for (size_t i = 0; i < initialShapeCount; i++) {
		const InitialShapeWrapper& isw = *ffiInitialShapes[i];
		isb->setGeometry(isw.vertexCoords, isw.vertexCoordsCount, isw.indices, isw.indicesCount, isw.faceCounts,
		                 isw.faceCountsCount);
		initialShapeAttributes.emplace_back(amb->createAttributeMapAndReset()); // TODO
		resolveMaps.emplace_back(rmb->createResolveMapAndReset());              // TODO

		isw.setAttributes(*isb, initialShapeAttributes.back().get(), resolveMaps.back().get());

		initialShapes.emplace_back(isb->createInitialShapeAndReset());
		initialShapePtrs[i] = initialShapes.back().get();
	}

	AttributeMapUPtr validatedEncOpts(createValidatedOptions(encoders[0]));
	std::vector<const prt::AttributeMap*> tmpEncoderOpts{validatedEncOpts.get()};

	auto callbacksBinding = std::make_unique<RustCallbacksBinding>(callbacks);
	prt::Status status =
	        prt::generate(initialShapePtrs.data(), initialShapePtrs.size(), occlusionHandles, encoders, encodersCount,
	                      tmpEncoderOpts.data(), callbacksBinding.get(), cache, occlSet, generateOptions);
	return status;
}

namespace {

using HandlerHolder = std::map<void*, std::unique_ptr<RustLogHandlerBinding>>;
HandlerHolder logHandlerHolder;

} // namespace

void ffi_add_log_handler(AbstractLogHandlerBinding* logHandler) {
	auto [handlerIt, done] =
	        logHandlerHolder.emplace(logHandler->context, std::make_unique<RustLogHandlerBinding>(logHandler));
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

// AttributeMapBuilder implementations
AttributeMapBuilderWrapper* ffi_create_attribute_map_builder() {
	prt::AttributeMapBuilder* builder = prt::AttributeMapBuilder::create();
	return reinterpret_cast<AttributeMapBuilderWrapper*>(builder);
}

void ffi_destroy_attribute_map_builder(AttributeMapBuilderWrapper* builder) {
	if (builder) {
		prt::AttributeMapBuilder* prtBuilder = reinterpret_cast<prt::AttributeMapBuilder*>(builder);
		prtBuilder->destroy();
	}
}

prt::Status ffi_attribute_map_builder_set_string(AttributeMapBuilderWrapper* builder, const char* key, const char* value) {
	if (!builder || !key || !value) return prt::STATUS_ILLEGAL_VALUE;
	
	prt::AttributeMapBuilder* prtBuilder = reinterpret_cast<prt::AttributeMapBuilder*>(builder);
	std::wstring wKey = toUTF16FromUTF8(key);
	std::wstring wValue = toUTF16FromUTF8(value);
	return prtBuilder->setString(wKey.c_str(), wValue.c_str());
}

prt::Status ffi_attribute_map_builder_set_float(AttributeMapBuilderWrapper* builder, const char* key, double value) {
	if (!builder || !key) return prt::STATUS_ILLEGAL_VALUE;
	
	prt::AttributeMapBuilder* prtBuilder = reinterpret_cast<prt::AttributeMapBuilder*>(builder);
	std::wstring wKey = toUTF16FromUTF8(key);
	return prtBuilder->setFloat(wKey.c_str(), value);
}

prt::Status ffi_attribute_map_builder_set_int(AttributeMapBuilderWrapper* builder, const char* key, int32_t value) {
	if (!builder || !key) return prt::STATUS_ILLEGAL_VALUE;
	
	prt::AttributeMapBuilder* prtBuilder = reinterpret_cast<prt::AttributeMapBuilder*>(builder);
	std::wstring wKey = toUTF16FromUTF8(key);
	return prtBuilder->setInt(wKey.c_str(), value);
}

prt::Status ffi_attribute_map_builder_set_bool(AttributeMapBuilderWrapper* builder, const char* key, bool value) {
	if (!builder || !key) return prt::STATUS_ILLEGAL_VALUE;
	
	prt::AttributeMapBuilder* prtBuilder = reinterpret_cast<prt::AttributeMapBuilder*>(builder);
	std::wstring wKey = toUTF16FromUTF8(key);
	return prtBuilder->setBool(wKey.c_str(), value);
}

const AttributeMapWrapper* ffi_attribute_map_builder_create_map(AttributeMapBuilderWrapper* builder) {
	if (!builder) return nullptr;
	
	prt::AttributeMapBuilder* prtBuilder = reinterpret_cast<prt::AttributeMapBuilder*>(builder);
	const prt::AttributeMap* map = prtBuilder->createAttributeMapAndReset();
	return reinterpret_cast<const AttributeMapWrapper*>(map);
}

// ResolveMapBuilder implementations
ResolveMapBuilderWrapper* ffi_create_resolve_map_builder() {
	prt::ResolveMapBuilder* builder = prt::ResolveMapBuilder::create();
	return reinterpret_cast<ResolveMapBuilderWrapper*>(builder);
}

void ffi_destroy_resolve_map_builder(ResolveMapBuilderWrapper* builder) {
	if (builder) {
		prt::ResolveMapBuilder* prtBuilder = reinterpret_cast<prt::ResolveMapBuilder*>(builder);
		prtBuilder->destroy();
	}
}

prt::Status ffi_resolve_map_builder_add_entry(ResolveMapBuilderWrapper* builder, const char* key, const char* value) {
	if (!builder || !key || !value) return prt::STATUS_ILLEGAL_VALUE;
	
	prt::ResolveMapBuilder* prtBuilder = reinterpret_cast<prt::ResolveMapBuilder*>(builder);
	std::wstring wKey = toUTF16FromUTF8(key);
	std::wstring wValue = toUTF16FromUTF8(value);
	return prtBuilder->addEntry(wKey.c_str(), wValue.c_str());
}

const ResolveMapWrapper* ffi_resolve_map_builder_create_map(ResolveMapBuilderWrapper* builder) {
	if (!builder) return nullptr;
	
	prt::ResolveMapBuilder* prtBuilder = reinterpret_cast<prt::ResolveMapBuilder*>(builder);
	const prt::ResolveMap* map = prtBuilder->createResolveMapAndReset();
	return reinterpret_cast<const ResolveMapWrapper*>(map);
}

// EncoderInfo/DecoderInfo implementations
const char* const* ffi_list_encoder_ids(size_t* count) {
	if (!count) return nullptr;
	
	// First call to get required buffer size
	size_t bufferSize = 0;
	prt::listEncoderIds(nullptr, &bufferSize);
	if (bufferSize == 0) {
		*count = 0;
		return nullptr;
	}
	
	// Allocate buffer and get the list
	static std::vector<wchar_t> wideBuffer;
	wideBuffer.resize(bufferSize);
	prt::listEncoderIds(wideBuffer.data(), &bufferSize);
	
	// Convert to UTF-8 and store as static (simplified approach)
	static std::string encoderList = toUTF8FromUTF16(wideBuffer.data());
	static std::vector<const char*> encoderPtrs;
	
	// Parse semicolon-separated list
	encoderPtrs.clear();
	const char* start = encoderList.c_str();
	const char* end = encoderList.c_str() + encoderList.length();
	while (start < end) {
		const char* semicolon = std::strchr(start, ';');
		if (semicolon == nullptr) semicolon = end;
		encoderPtrs.push_back(start);
		if (semicolon < end) {
			const_cast<char*>(semicolon)[0] = '\0'; // null-terminate
			start = semicolon + 1;
		} else {
			break;
		}
	}
	
	*count = encoderPtrs.size();
	return encoderPtrs.data();
}

const char* const* ffi_list_decoder_ids(size_t* count) {
	if (!count) return nullptr;
	
	// First call to get required buffer size  
	size_t bufferSize = 0;
	prt::listDecoderIds(nullptr, &bufferSize);
	if (bufferSize == 0) {
		*count = 0;
		return nullptr;
	}
	
	// Allocate buffer and get the list
	static std::vector<wchar_t> wideBuffer;
	wideBuffer.resize(bufferSize);
	prt::listDecoderIds(wideBuffer.data(), &bufferSize);
	
	// Convert to UTF-8 and store as static (simplified approach)
	static std::string decoderList = toUTF8FromUTF16(wideBuffer.data());
	static std::vector<const char*> decoderPtrs;
	
	// Parse semicolon-separated list
	decoderPtrs.clear();
	const char* start = decoderList.c_str();
	const char* end = decoderList.c_str() + decoderList.length();
	while (start < end) {
		const char* semicolon = std::strchr(start, ';');
		if (semicolon == nullptr) semicolon = end;
		decoderPtrs.push_back(start);
		if (semicolon < end) {
			const_cast<char*>(semicolon)[0] = '\0'; // null-terminate
			start = semicolon + 1;
		} else {
			break;
		}
	}
	
	*count = decoderPtrs.size();
	return decoderPtrs.data();
}

const EncoderInfoWrapper* ffi_create_encoder_info(const char* encoder_id) {
	if (!encoder_id) return nullptr;
	
	std::wstring wEncoderId = toUTF16FromUTF8(encoder_id);
	const prt::EncoderInfo* info = prt::createEncoderInfo(wEncoderId.c_str());
	return reinterpret_cast<const EncoderInfoWrapper*>(info);
}

void ffi_destroy_encoder_info(const EncoderInfoWrapper* encoder_info) {
	if (encoder_info) {
		const prt::EncoderInfo* prtInfo = reinterpret_cast<const prt::EncoderInfo*>(encoder_info);
		prtInfo->destroy();
	}
}

const char* ffi_encoder_info_get_id(const EncoderInfoWrapper* encoder_info) {
	if (!encoder_info) return nullptr;
	
	const prt::EncoderInfo* prtInfo = reinterpret_cast<const prt::EncoderInfo*>(encoder_info);
	const wchar_t* wideId = prtInfo->getID();
	static std::string id = toUTF8FromUTF16(wideId);
	return id.c_str();
}

const char* ffi_encoder_info_get_name(const EncoderInfoWrapper* encoder_info) {
	if (!encoder_info) return nullptr;
	
	const prt::EncoderInfo* prtInfo = reinterpret_cast<const prt::EncoderInfo*>(encoder_info);
	const wchar_t* wideName = prtInfo->getName();
	static std::string name = toUTF8FromUTF16(wideName);
	return name.c_str();
}

const char* ffi_encoder_info_get_description(const EncoderInfoWrapper* encoder_info) {
	if (!encoder_info) return nullptr;
	
	const prt::EncoderInfo* prtInfo = reinterpret_cast<const prt::EncoderInfo*>(encoder_info);
	const wchar_t* wideDesc = prtInfo->getDescription();
	static std::string desc = toUTF8FromUTF16(wideDesc);
	return desc.c_str();
}

// StringUtils implementations
prt::Status ffi_string_utils_percent_encode(const char* input, char* output, size_t* size) {
	if (!input || !size) return prt::STATUS_ILLEGAL_VALUE;
	
	prt::Status status;
	prt::StringUtils::percentEncode(input, output, size, &status);
	return status;
}

