// beam_glass.h, part 2 (cut there by size, as it was): read only through beam_glass.h, in order.
#pragma once

namespace beam {

// One landed patch: a convex polygon on the floor (x, z), flat colour.
struct Patch {
    static constexpr int SEQ = 24;
    int first = 0, count = 0;  // corners in Result::xz
    V3 rgb{0, 0, 0};           // linear sRGB irradiance, in units of the sun's
    int band = 0;
    uint16_t seq[SEQ] = {};    // the way the light took (see step())
    uint8_t len = 0;
    uint64_t path = 0;         // hash of the above
};
struct Result {
    std::vector<double> xz;      // x0, z0, x1, z1, ...
    std::vector<double> travel;  // per corner: distance flown from the glass to the floor
    std::vector<Patch> patches;
    // Bookkeeping per unit of sun irradiance, summed over bands with each band counted as 1 / bands:
    double fluxIn = 0, fluxFloor = 0, fluxSky = 0, fluxDropped = 0;
    int beams = 0;
};

namespace detail {
// Helpers that are started once and then wait for work. (Starting threads anew for every frame cost nothing on a
// quiet machine and stalled for a tenth of a second now and then on a busy one.)
class Workers {
  public:
    static Workers& get() {
        static Workers w;
        return w;
    }
    // call(ctx, i) for every i below n, on the calling thread and up to threads - 1 helpers
    void run(size_t n, unsigned threads, void (*call)(void*, size_t), void* ctx) {
        static thread_local bool inside = false;
        std::unique_lock<std::mutex> one(turn_, std::defer_lock);
        if (inside || threads <= 1 || n <= 1 || pool_.empty() || !one.try_lock()) {
            // already inside a run (or another thread has the helpers): do it here
            for (size_t i = 0; i < n; ++i) call(ctx, i);
            return;
        }
        inside = true;
        {
            std::lock_guard<std::mutex> lock(m_);
            call_ = call;
            ctx_ = ctx;
            n_ = n;
            next_.store(0);
            wanted_ = (unsigned)std::min<size_t>({(size_t)threads - 1, pool_.size(), n - 1});
            ++round_;
        }
        wake_.notify_all();
        for (;;) {
            const size_t i = next_.fetch_add(1);
            if (i >= n) break;
            call(ctx, i);
        }
        {
            std::unique_lock<std::mutex> lock(m_);
            wanted_ = 0;  // helpers that have not woken yet are not needed any more
            done_.wait(lock, [&] { return busy_ == 0; });
        }
        inside = false;
    }

  private:
    Workers() {
        const unsigned count = std::max(1u, std::thread::hardware_concurrency()) - 1;
        for (unsigned w = 0; w < count; ++w) pool_.emplace_back([this] { loop(); });
    }
    ~Workers() {
        {
            std::lock_guard<std::mutex> lock(m_);
            quit_ = true;
        }
        wake_.notify_all();
        for (std::thread& t : pool_) t.join();
    }
    void loop() {
#ifdef __APPLE__
        // (helpers of default priority are woken late and pushed aside when the machine is busy: WORKQOS=1 asks the
        // system to treat them as part of what the user is looking at)
        if (std::getenv("WORKQOS")) pthread_set_qos_class_self_np(QOS_CLASS_USER_INTERACTIVE, 0);
#endif
        std::unique_lock<std::mutex> lock(m_);
        uint64_t seen = 0;
        for (;;) {
            wake_.wait(lock, [&] { return quit_ || (round_ != seen && wanted_ > 0); });
            if (quit_) return;
            seen = round_;
            --wanted_;
            ++busy_;
            void (*call)(void*, size_t) = call_;
            void* ctx = ctx_;
            const size_t n = n_;
            lock.unlock();
            for (;;) {
                const size_t i = next_.fetch_add(1);
                if (i >= n) break;
                call(ctx, i);
            }
            lock.lock();
            if (--busy_ == 0) done_.notify_all();
        }
    }
    std::mutex m_, turn_;
    std::condition_variable wake_, done_;
    std::vector<std::thread> pool_;
    void (*call_)(void*, size_t) = nullptr;
    void* ctx_ = nullptr;
    size_t n_ = 0;
    std::atomic<size_t> next_{0};
    unsigned wanted_ = 0, busy_ = 0;
    uint64_t round_ = 0;
    bool quit_ = false;
};
}  // namespace detail
template <class F>
inline void parallel_for(size_t n, unsigned threads, F&& f) {
    if (threads <= 1 || n <= 1) {
        for (size_t i = 0; i < n; ++i) f(i);
        return;
    }
    detail::Workers::get().run(n, threads, [](void* ctx, size_t i) { (*static_cast<std::remove_reference_t<F>*>(ctx))(i); }, (void*)&f);
}

}  // namespace beam
