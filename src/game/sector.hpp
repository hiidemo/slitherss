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

/**
 * http://stackoverflow.com/questions/563198/how-do-you-detect-where-two-line-segments-intersect
 *
 * FWIW, the following function (in C) both detects line intersections and determines the intersection point.
 * It is based on an algorithm in Andre LeMothe's "Tricks of the Windows Game Programming Gurus".
 * It's not dissimilar to some of the algorithm's in other answers (e.g. Gareth's).
 * LeMothe then uses Cramer's Rule (don't ask me) to solve the equations themselves.
 *
 * I can attest that it works in my feeble asteroids clone, and seems to deal correctly with the edge cases
 * described in other answers by Elemental, Dan and Wodzu. It's also probably faster than the code posted by
 * KingNestor because it's all multiplication and division, no square roots!
 */
bool intersect_segments(float p0_x, float p0_y, float p1_x, float p1_y,
                        float p2_x, float p2_y, float p3_x, float p3_y);

// center, point, radius
bool intersect_circle(float c_x, float c_y, float p_x, float p_y, float r);

// line vw, and point p
// http://stackoverflow.com/questions/849211/shortest-distance-between-a-point-and-a-line-segment
float distance_squared(float v_x, float v_y, float w_x, float w_y, float p_x, float p_y);

// points p0, p1
float distance_squared(float p0_x, float p0_y, float p1_x, float p1_y);

// https://en.wikipedia.org/wiki/Methods_of_computing_square_roots
float fastsqrt(float val);

// float fastinvsqrt(float x);

struct snake_bb_pos {
    float x;
    float y;
    float r; // squared radius

    snake_bb_pos() = default;
    snake_bb_pos(const snake_bb_pos &p) : x(p.x), y(p.y), r(p.r) {}
    snake_bb_pos(float in_x, float in_y, float in_r) : x(in_x), y(in_y), r(in_r) {}

    inline bool intersect(const snake_bb_pos &bb2) const {
        const float dx = x - bb2.x;
        const float dy = y - bb2.y;
        const float r2 = r + bb2.r;
        return dx * dx + dy * dy <= r2 * r2;
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

    bool remove_sector(const std::vector<sector *>::iterator &i);
    bool binary_search(sector *s);
    void sort();

    size_t get_sectors_count();
    size_t get_snakes_in_sectors_count();
    void reg_new_sector_if_missing(sector *s);
    void reg_old_sector_if_missing(sector *s);
};

struct sector {
    uint8_t x;
    uint8_t y;

    snake_bb_pos box;

    std::vector<snake_bb *> m_snakes;
    std::vector<food> m_food;

    sector(uint8_t in_x, uint8_t in_y) : x(in_x), y(in_y) {
        static const uint16_t half = world_config::sector_size / 2;
        static constexpr float r = world_config::sector_diag_size / 2.0f;

        box = {
            1.0f * (world_config::sector_size * x + half),
            1.0f * (world_config::sector_size * y + half),
            r };
    }

    inline bool intersect(const snake_bb_pos &box2) const {
        return box.intersect(box2);
    }

    void remove_snake(snake_id_t id);
};

class sectors : public std::vector<sector> {
public:
    sectors() : std::vector<sector>() { }

    void init_sectors();
    size_t get_index(uint16_t x, uint16_t y);
    sector *get_sector(uint16_t x, uint16_t y);
};

#endif //SLITHER_GAME_SECTOR_HPP
