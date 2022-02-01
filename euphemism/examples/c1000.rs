use euphemism::{Cluster, Sample};

fn main() {
    let labels = include_str!("../../meals.txt").lines().take(1000);

    let mut clusters = Vec::<Cluster>::new();

    for label in labels {
        let sample = Sample::new(label);

        let best = clusters
            .iter_mut()
            .max_by(|a, b| a.score(&sample).partial_cmp(&b.score(&sample)).unwrap());

        match best {
            Some(cluster) if cluster.score(&sample) > 0.6 => {
                cluster.samples.push(sample);
            }
            _ => clusters.push(Cluster::with_samples(vec![sample])),
        };
    }

    for cluster in clusters {
        println!("{}", cluster.label().unwrap());

        for sample in cluster.samples {
            println!("- {}", sample.label());
        }
    }
}
