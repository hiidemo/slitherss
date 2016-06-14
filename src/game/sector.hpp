#ifndef SLITHER_GAME_SECTOR_HPP
#define SLITHER_GAME_SECTOR_HPP

#include "food.hpp"
#include "config.hpp"

#include <cstddef>
#include <cstdint>
#include <vector>
#include <cmath>

struct snake;
struct sector;

struct snake_bb_pos {
    float x;
    float y;
    float r2; // squared radius

    snake_bb_pos() = default;
    snake_bb_pos(const snake_bb_pos &p) : x(p.x), y(p.y), r2(p.r2) {}
    snake_bb_pos(float in_x, float in_y, float in_r2) : x(in_x), y(in_y), r2(in_r2) {}

    inline bool intersect(const snake_bb_pos &bb2) const {
        const float dx = x - bb2.x;
        const float dy = y - bb2.y;
        // x^2 + y^2 <= r^2 = (r1 + r2) ^ 2 <= r1^2 + r2^2 + 2*r1*r2 <= r1^2 + r2^2 + 2*max(r1^2,r2^2)
        const float rr = fmaxf(r2, bb2.r2);
        return dx * dx + dy * dy <= r2 + bb2.r2 + 2.0f * rr;
    }
};

struct snake_bb : snake_bb_pos {
    snake_id_t id;
    const snake * snake_ptr;
    std::vector<sector *> sectors;
    std::vector<sector *> new_sectors;
    std::vector<sector *> old_sectors;

    snake_bb() = default;
    snake_bb(snake_bb_pos in_pos, uint16_t in_id, const snake * in_ptr, std::vector<sector *> in_sectors) :
        snake_bb_pos(in_pos), id(in_id), snake_ptr(in_ptr), sectors(in_sectors) {}

    bool find(sector *s);
    size_t get_sectors_count();
    size_t get_snakes_in_sectors_count();
    void reg_new_sector_if_missing(sector *s);
    void reg_old_sector_if_missing(sector *s);
};

struct sector {
    uint8_t x;
    uint8_t y;

    std::vector<snake_bb> m_snakes;
    std::vector<food> m_food;

    inline bool intersect(const snake_bb_pos &bb2) const {
        static const uint16_t half = world_config::sector_size / 2;
        static constexpr float r_sqr = 1.0f * world_config::sector_diag_size * world_config::sector_diag_size;

        const snake_bb_pos bb1 = {
                1.0f * (world_config::sector_size * x + half),
                1.0f * (world_config::sector_size * y + half),
                r_sqr };
        return bb1.intersect(bb2);
    }

    void remove_snake(snake_id_t id);
};

class sectors : public std::vector<sector> {
public:
    sectors() : std::vector<sector>() { }

    void init_sectors(const uint16_t sector_count_along_edge);
    size_t get_index(uint16_t x, uint16_t y);
    sector *get_sector(uint16_t x, uint16_t y);

private:

    uint16_t width = 0;
};

#endif //SLITHER_GAME_SECTOR_HPP
