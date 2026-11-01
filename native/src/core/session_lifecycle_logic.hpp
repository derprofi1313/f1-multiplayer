#pragma once
#include <cstdint>

namespace runtime::session_life {

constexpr int64_t END_EXITED = 0;
constexpr int64_t END_RESTART = 1;
constexpr int64_t END_UNLOAD = 2;

struct Emit {
    bool end = false;
    bool unload = false;
    int64_t reason = END_EXITED;
};

class Lifecycle {
public:
    bool on_start()
    {
        bool restart = open_;
        open_ = true;
        seen_ = true;
        return restart;
    }

    Emit on_session_exiting()
    {
        Emit e;
        if (open_) {
            open_ = false;
            e.end = true;
        }
        return e;
    }

    Emit on_unload_entering()
    {
        Emit e = on_session_exiting();
        if (e.end)
            e.reason = END_UNLOAD;
        if (seen_) {
            seen_ = false;
            e.unload = true;
        }
        return e;
    }

    bool open() const { return open_; }

private:
    bool open_ = false;
    bool seen_ = false;
};

}
