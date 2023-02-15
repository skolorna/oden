SELECT
  meals.date,
  meals.meal,
  AVG(rating)::FLOAT4 AS rating,
  COUNT(rating) AS reviews
FROM
  meals
  LEFT JOIN reviews ON reviews.meal = meals.meal
  AND reviews.menu_id = meals.menu_id
WHERE
  meals.menu_id = $1
  AND $2::DATERANGE @> meals.date
GROUP BY
  meals.date,
  meals.meal
ORDER BY
  meals.date ASC
