//
//  DetailView.swift
//  Picosso
//
//  Created by Ritam Sarmah on 3/23/23.
//

import SwiftUI

struct DetailView: View {
    
    @Environment(\.dismiss) var dismiss
    
    @ObservedObject var viewModel: DetailViewModel
    
    var body: some View {
        HStack {
            Spacer()
            VStack(spacing: 4) {
                Text(viewModel.artwork.title)
                    .font(.title)
                    .fontWeight(.bold)
                    .multilineTextAlignment(.center)
                Text(viewModel.artwork.artist)
                
                Spacer()
                
                AsyncImage(url: viewModel.imageURL) { image in
                    image
                        .resizable()
                        .aspectRatio(contentMode: .fit)
                        .frame(maxHeight: 500)
                } placeholder: {
                    ProgressView()
                        .tint(viewModel.artwork.dark ? .white : .gray)
                }
                
                Spacer()
            }
            Spacer()
        }
        .foregroundColor(viewModel.artwork.dark ? Color.white : Color.black)
        .background(viewModel.artwork.dark ? Color.black : Color.white)
    }
}

//struct DetailView_Previews: PreviewProvider {
//    static var previews: some View {
//        UploadView()
//    }
//}
