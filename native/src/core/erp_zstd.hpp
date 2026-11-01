#pragma once

#include <zstd.h>

#include <string>
#include <utility>

// The zstd frames the core writes into ERP fragments carry their content size and a window of at most 1 MiB,
// whatever the level, like the game's own archives. The game's stream decoder (0x1447b0a60) sizes its buffer from
// a frame's window and content size inside the fixed EGO_ERP_WORKSPACE heap, so wider windows run that heap out.
// tools/erpw.py and packages/formats write the same frames.

namespace swofty_core::erp_zstd {

inline constexpr int WINDOW_LOG = 20;

inline bool compress(const char *data, size_t n, int level, std::string &out)
{
    ZSTD_CCtx *cctx = ZSTD_createCCtx();
    if (!cctx)
        return false;
    const std::pair<ZSTD_cParameter, int> params[] = {
        {ZSTD_c_compressionLevel, level}, {ZSTD_c_windowLog, WINDOW_LOG}, {ZSTD_c_contentSizeFlag, 1}};
    size_t r = 0;
    for (auto &[p, v] : params)
        if (ZSTD_isError(r = ZSTD_CCtx_setParameter(cctx, p, v)))
            break;
    if (!ZSTD_isError(r)) {
        out.resize(ZSTD_compressBound(n));
        r = ZSTD_compress2(cctx, out.data(), out.size(), data, n);
    }
    ZSTD_freeCCtx(cctx);
    if (ZSTD_isError(r))
        return false;
    out.resize(r);
    return true;
}

} // namespace swofty_core::erp_zstd
