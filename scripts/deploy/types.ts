export type Trait = {
  display_type?: string;
  trait_type: string;
  value: string;
};

export type Metadata = {
  image?: string;
  image_data?: string;
  external_url?: string;
  description?: string;
  name?: string;
  attributes?: Trait[];
  background_color?: string;
  animation_url?: string;
  youtube_url?: string;
};
