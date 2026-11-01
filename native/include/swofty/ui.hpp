#pragma once
#include <cstdint>
#include <memory>
#include <string>

namespace swofty::ui {

struct Command {
    uint32_t hash;
    const char *name;
    const void *data;
};

class ViewModel {
public:
    virtual ~ViewModel() = default;
    virtual void on_activated() {}
    virtual void on_deactivated() {}
    virtual void on_bound() {}
    virtual bool on_command(const Command &) { return false; }
    virtual void update(float) {}

    const std::string &screen() const { return screen_; }
    const std::string &path() const { return path_; }

    bool set_text(const char *prop, const char *utf8);
    bool set_bool(const char *prop, bool value);
    bool set_int(const char *prop, int32_t value);

private:
    friend struct Host;
    std::string screen_, path_;
    void *native_ = nullptr;
};

using Factory = std::unique_ptr<ViewModel> (*)(const std::string &screen);

void register_view_model(const char *screen_prefix, Factory factory);

}

#define SWOFTY_VIEW_MODEL_CAT_(a, b) a##b
#define SWOFTY_VIEW_MODEL_CAT(a, b) SWOFTY_VIEW_MODEL_CAT_(a, b)
#define SWOFTY_VIEW_MODEL(prefix, Class)                                                                              \
    namespace {                                                                                                       \
    const int SWOFTY_VIEW_MODEL_CAT(swofty_vm_reg_, __LINE__) = (::swofty::ui::register_view_model(                   \
                                                                     prefix,                                          \
                                                                     [](const std::string &) -> std::unique_ptr<::swofty::ui::ViewModel> { \
                                                                         return std::make_unique<Class>();            \
                                                                     }),                                              \
                                                                 0);                                                  \
    }
